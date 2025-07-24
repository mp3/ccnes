#!/bin/bash

# Build the WebAssembly module
echo "Building CCNES WebAssembly module..."

# Build with wasm-pack
wasm-pack build --target web --out-dir pkg

# Create a simple HTTP server script
cat > serve.py << 'EOF'
#!/usr/bin/env python3
import http.server
import socketserver
import os

class MyHTTPRequestHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        super().end_headers()

    def guess_type(self, path):
        mimetype = super().guess_type(path)
        if path.endswith('.wasm'):
            return 'application/wasm'
        return mimetype

os.chdir(os.path.dirname(os.path.abspath(__file__)))

PORT = 8000
with socketserver.TCPServer(("", PORT), MyHTTPRequestHandler) as httpd:
    print(f"Server running at http://localhost:{PORT}/")
    print("Press Ctrl+C to stop")
    httpd.serve_forever()
EOF

chmod +x serve.py

echo "Build complete!"
echo "To run the emulator:"
echo "  1. Run './serve.py' to start the web server"
echo "  2. Open http://localhost:8000 in your browser"
echo "  3. Click 'Load ROM' to load a .nes file"