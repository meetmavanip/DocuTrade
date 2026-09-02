from http.server import ThreadingHTTPServer, SimpleHTTPRequestHandler
import os
import sys

DIRECTORY = os.path.abspath(os.path.join(os.path.dirname(__file__), 'frontend'))

class CleanUrlHandler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=DIRECTORY, **kwargs)

    def translate_path(self, path):
        translated = super().translate_path(path)
        if not os.path.exists(translated):
            candidate = translated + ".html"
            if os.path.isfile(candidate):
                return candidate
        return translated

    def end_headers(self):
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Cache-Control', 'no-cache, no-store, must-revalidate')
        super().end_headers()

if __name__ == '__main__':
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8080
    with ThreadingHTTPServer(("", port), CleanUrlHandler) as httpd:
        print(f"DocuTrade Frontend Server running on http://localhost:{port}")
        sys.stdout.flush()
        httpd.serve_forever()
