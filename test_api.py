import urllib.request
import urllib.parse
import json
import random
import string

def rstr(n):
    return ''.join(random.choices(string.ascii_letters, k=n))

base_url = "http://localhost:3000/api"

# 1. Register
email = f"test_{rstr(5)}@example.com"
req = urllib.request.Request(
    f"{base_url}/auth/register",
    data=json.dumps({
        "email": email,
        "password": "Password123!",
        "name": "Test User",
        "organization_name": "Test Org",
        "role": "exporter"
    }).encode('utf-8'),
    headers={"Content-Type": "application/json"}
)
try:
    with urllib.request.urlopen(req) as response:
        res_data = json.loads(response.read().decode())
        token = res_data['token']
        print(f"Token: {token}")
except urllib.error.HTTPError as e:
    print(f"Register Error: {e.read().decode()}")
    exit(1)

# 2. Get Shipments
req2 = urllib.request.Request(
    f"{base_url}/shipments",
    headers={"Authorization": f"Bearer {token}"}
)
try:
    with urllib.request.urlopen(req2) as response:
        print(response.read().decode())
except urllib.error.HTTPError as e:
    print(f"Shipments Error: {e.read().decode()}")
