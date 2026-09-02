import urllib.request, urllib.parse, json

base_url = 'http://localhost:3000/api'

req_login = urllib.request.Request(
    base_url + '/auth/login',
    data=json.dumps({'email': 'seller_debug@example.com', 'password': 'Password123!'}).encode('utf-8'),
    headers={'Content-Type': 'application/json'}
)
with urllib.request.urlopen(req_login) as resp:
    data = json.loads(resp.read().decode())
    token = data['token']
    print("Logged in successfully, token received.")

req_ships = urllib.request.Request(base_url + '/shipments', headers={'Authorization': 'Bearer ' + token})
with urllib.request.urlopen(req_ships) as resp:
    ships = json.loads(resp.read().decode())
    print('Existing shipments count:', len(ships.get('shipments', [])))

shipment_id = None
if ships.get('shipments') and len(ships['shipments']) > 0:
    shipment_id = ships['shipments'][0]['id']
else:
    req_create = urllib.request.Request(
        base_url + '/shipments',
        data=json.dumps({
            'origin_country': 'India',
            'origin_location': 'Mumbai Port',
            'destination_country': 'UAE',
            'destination_location': 'Dubai',
            'buyer_name': 'Dubai Imports',
            'incoterms': 'FOB',
            'currency': 'USD',
            'total_value': 50000.0,
            'products': []
        }).encode('utf-8'),
        headers={'Content-Type': 'application/json', 'Authorization': 'Bearer ' + token}
    )
    with urllib.request.urlopen(req_create) as resp:
        res = json.loads(resp.read().decode())
        print('Created shipment:', res)
        shipment_id = res.get('shipment_id') or res.get('id')

print('Testing upload for shipment:', shipment_id)

boundary = '----Boundary12345'
parts = [
    f'--{boundary}',
    'Content-Disposition: form-data; name="shipment_id"',
    '',
    str(shipment_id),
    f'--{boundary}',
    'Content-Disposition: form-data; name="type"',
    '',
    'Bill of Lading',
    f'--{boundary}',
    'Content-Disposition: form-data; name="hash"',
    '',
    'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
    f'--{boundary}',
    'Content-Disposition: form-data; name="file"; filename="bol.pdf"',
    'Content-Type: application/pdf',
    '',
    '%PDF-1.4 sample pdf content',
    f'--{boundary}--',
    ''
]
body = '\r\n'.join(parts).encode('utf-8')

req_upload = urllib.request.Request(
    base_url + '/documents/upload',
    data=body,
    headers={
        'Content-Type': 'multipart/form-data; boundary=' + boundary,
        'Authorization': 'Bearer ' + token
    }
)
try:
    with urllib.request.urlopen(req_upload) as resp:
        print('Upload response:', resp.read().decode())
except urllib.error.HTTPError as e:
    print('Upload HTTP Error:', e.code, e.read().decode())

# Now test list_documents
req_docs = urllib.request.Request(base_url + '/documents', headers={'Authorization': 'Bearer ' + token})
try:
    with urllib.request.urlopen(req_docs) as resp:
        print('List documents status:', resp.status)
        print('List documents response:', resp.read().decode())
except urllib.error.HTTPError as e:
    print('List documents HTTP Error:', e.code, e.read().decode())
