import init, { generate_layout } from "/pkg/wasm_imager.js";

self.onmessage = async (e) => {
  const { schema, filesPayload, outputName } = e.data;

  try {
    await init();
    console.log("[Worker] Wasm init success");

    const fileSizes = {};
    for (const id in filesPayload) {
      if (filesPayload[id]) {
        fileSizes[id] = filesPayload[id].size;
      }
    }

    const layoutResult = generate_layout(schema, fileSizes);
    console.log("[Worker] Wasm layouting success:", layoutResult);

    const root = await navigator.storage.getDirectory();
    await root.removeEntry(outputName).catch(() => {});
    
    const fileHandle = await root.getFileHandle(outputName, { create: true });
    const syncHandle = await fileHandle.createSyncAccessHandle();
    
    // truncate to disk size
    syncHandle.truncate(Number(layoutResult.disk_size));

    // write metadata chunks
    for (const chunk of layoutResult.metadata_chunks) {
      const dataView = new Uint8Array(chunk.data);
      syncHandle.write(dataView, { at: Number(chunk.offset) });
    }

    // setup a 4MB buffer for writing
    const BUFFER_SIZE = 4 * 1024 * 1024;

    for (const resolved of layoutResult.resolved_partitions) {
      const fileBlob = filesPayload[resolved.id];
      if (!fileBlob) {
        console.log(`[Worker] skip empty partition: ${resolved.id}`);
        continue;
      }

      console.log(`[Worker] start writing [ID: ${resolved.id}] -> target offset: ${resolved.offset} bytes`);

      const reader = fileBlob.stream().getReader();
      let currentOffset = Number(resolved.offset);
      
      let buffer = new Uint8Array(BUFFER_SIZE);
      let bufLen = 0;

      while (true) {
        const { done, value } = await reader.read();
        
        if (value) {
          let valueOffset = 0;
          while (valueOffset < value.length) {
            const space = BUFFER_SIZE - bufLen;
            const chunkToCopy = Math.min(space, value.length - valueOffset);
            
            // concatenate data into buffer
            buffer.set(value.subarray(valueOffset, valueOffset + chunkToCopy), bufLen);
            bufLen += chunkToCopy;
            valueOffset += chunkToCopy;
            
            // if buffer is full, write it to OPFS and reset
            if (bufLen === BUFFER_SIZE) {
              syncHandle.write(buffer, { at: currentOffset });
              currentOffset += BUFFER_SIZE;
              bufLen = 0;
            }
          }
        }

        if (done) {
          // if stream is done, write the remaining buffer to OPFS
          if (bufLen > 0) {
            syncHandle.write(buffer.subarray(0, bufLen), { at: currentOffset });
            currentOffset += bufLen;
            bufLen = 0;
          }
          break;
        }
      }
    }

    syncHandle.flush();
    syncHandle.close();

    self.postMessage({ status: "done", fileName: outputName });
  } catch (err) {
    console.error(err);
    self.postMessage({
      status: "error",
      error: err.message ? err.message : String(err),
    });
  }
};
