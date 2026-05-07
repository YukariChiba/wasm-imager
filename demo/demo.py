import http.server
import socketserver

PORT = 8080

class CORSRequestHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        # CORS headers to allow cross-origin requests
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        super().end_headers()

print(f"Local server started. Access it at: http://localhost:{PORT}")
with socketserver.TCPServer(("", PORT), CORSRequestHandler) as httpd:
    httpd.serve_forever()
