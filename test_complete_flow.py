import urllib.request, urllib.parse, json
import uuid

base_url = 'http://localhost:3000/api'

def run_test():
    print("========================================")
    print("STARTING COMPLETE VERIFICATION TEST")
    print("========================================")
    
    # 1. Register or Login as Seller
    seller_email = "seller_real@example.com"
    seller_pw = "Password123!"
    req_reg = urllib.request.Request(
        f'{base_url}/auth/register',
        data=json.dumps({
            'email': seller_email,
            'password': seller_pw,
            'name': 'Real Seller',
            'organization': 'Apex Exporters Inc',
            'role': 'SELLER'
        }).encode('utf-8'),
        headers={'Content-Type': 'application/json'}
    )
    try:
        with urllib.request.urlopen(req_reg) as resp:
            data = json.loads(resp.read().decode())
            seller_token = data['token']
            print("1. Seller Registered successfully:", data['user'])
    except Exception:
        req_login = urllib.request.Request(
            f'{base_url}/auth/login',
            data=json.dumps({'email': seller_email, 'password': seller_pw}).encode('utf-8'),
            headers={'Content-Type': 'application/json'}
        )
        with urllib.request.urlopen(req_login) as resp:
            data = json.loads(resp.read().decode())
            seller_token = data['token']
            print("1. Seller Logged in successfully:", data['user'])

    # 2. Register Buyer
    buyer_email = "buyer_real@example.com"
    buyer_pw = "Password123!"
    req_reg_buyer = urllib.request.Request(
        f'{base_url}/auth/register',
        data=json.dumps({
            'email': buyer_email,
            'password': buyer_pw,
            'name': 'Real Buyer',
            'organization': 'Global Buyers LLC',
            'role': 'BUYER'
        }).encode('utf-8'),
        headers={'Content-Type': 'application/json'}
    )
    try:
        with urllib.request.urlopen(req_reg_buyer) as resp:
            data = json.loads(resp.read().decode())
            buyer_token = data['token']
            print("2. Buyer Registered successfully:", data['user'])
    except Exception:
        req_login_buyer = urllib.request.Request(
            f'{base_url}/auth/login',
            data=json.dumps({'email': buyer_email, 'password': buyer_pw}).encode('utf-8'),
            headers={'Content-Type': 'application/json'}
        )
        with urllib.request.urlopen(req_login_buyer) as resp:
            data = json.loads(resp.read().decode())
            buyer_token = data['token']
            print("2. Buyer Logged in successfully:", data['user'])

    # 3. Seller creates a real Shipment
    req_create_ship = urllib.request.Request(
        f'{base_url}/shipments',
        data=json.dumps({
            'origin_country': 'India',
            'origin_location': 'Mundra Port, Gujarat',
            'destination_country': 'UAE',
            'destination_location': 'Jebel Ali Port, Dubai',
            'buyer_name': 'Global Buyers LLC',
            'incoterms': 'FOB',
            'currency': 'USD',
            'total_value': 125000.0,
            'products': [
                {'description': 'Industrial Valves', 'hs_code': '8481.80', 'quantity': 100, 'unit_price': 1250.0}
            ]
        }).encode('utf-8'),
        headers={'Content-Type': 'application/json', 'Authorization': f'Bearer {seller_token}'}
    )
    with urllib.request.urlopen(req_create_ship) as resp:
        ship_res = json.loads(resp.read().decode())
        shipment_id = ship_res['shipment_id']
        print(f"3. Created Shipment: {shipment_id}")

    # 4. Seller uploads a real PNG image document
    png_bytes = b'\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x06\x00\x00\x00\x1f\x15c4\x00\x00\x00\nIDATx\x9cc\x00\x01\x00\x00\x05\x00\x01\r\n-\xb4\x00\x00\x00\x00IEND\xaeB`\x82'
    import hashlib
    computed_hash = hashlib.sha256(png_bytes).hexdigest()
    
    boundary = f'----FormBoundary{uuid.uuid4().hex}'
    parts = [
        f'--{boundary}',
        'Content-Disposition: form-data; name="shipment_id"',
        '',
        shipment_id,
        f'--{boundary}',
        'Content-Disposition: form-data; name="type"',
        '',
        'Shipping Document',
        f'--{boundary}',
        'Content-Disposition: form-data; name="document_name"',
        '',
        'Bill of Lading Original.png',
        f'--{boundary}',
        'Content-Disposition: form-data; name="hash"',
        '',
        computed_hash,
        f'--{boundary}',
        'Content-Disposition: form-data; name="file"; filename="Bill_of_Lading.png"',
        'Content-Type: image/png',
        ''
    ]
    body_header = '\r\n'.join(parts).encode('utf-8') + b'\r\n' + png_bytes + f'\r\n--{boundary}--\r\n'.encode('utf-8')
    
    req_upload = urllib.request.Request(
        f'{base_url}/documents/upload',
        data=body_header,
        headers={
            'Content-Type': f'multipart/form-data; boundary={boundary}',
            'Authorization': f'Bearer {seller_token}'
        }
    )
    with urllib.request.urlopen(req_upload) as resp:
        upload_res = json.loads(resp.read().decode())
        doc_id = upload_res['document_id']
        server_hash = upload_res['hash']
        print(f"4. Document Uploaded: ID={doc_id}, Hash={server_hash}")
        assert server_hash == computed_hash, "Hash mismatch!"

    # 5. Seller lists documents
    req_seller_docs = urllib.request.Request(
        f'{base_url}/documents',
        headers={'Authorization': f'Bearer {seller_token}'}
    )
    with urllib.request.urlopen(req_seller_docs) as resp:
        docs_res = json.loads(resp.read().decode())
        print(f"5. Seller GET /documents: {len(docs_res['data'])} documents found.")
        uploaded_doc = next(d for d in docs_res['data'] if d['id'] == doc_id)
        print(f"   Status: {uploaded_doc['status']}, Type: {uploaded_doc['document_type']}")

    # 6. Verify file download
    req_file = urllib.request.Request(
        f'{base_url}/documents/{doc_id}/file',
        headers={'Authorization': f'Bearer {seller_token}'}
    )
    with urllib.request.urlopen(req_file) as resp:
        fetched_bytes = resp.read()
        assert fetched_bytes == png_bytes, "File content mismatch!"
        print(f"6. File Downloaded & Verified: {len(fetched_bytes)} bytes match original.")

    # 7. Check integrity endpoint
    req_integrity = urllib.request.Request(
        f'{base_url}/documents/{doc_id}/integrity',
        headers={'Authorization': f'Bearer {seller_token}'}
    )
    with urllib.request.urlopen(req_integrity) as resp:
        integ_res = json.loads(resp.read().decode())
        print(f"7. Document Integrity: Database match = {integ_res['database_integrity']}")
        assert integ_res['database_integrity'] == True, "Integrity check failed!"

    # 8. Buyer views and approves the document
    req_buyer_approve = urllib.request.Request(
        f'{base_url}/documents/{doc_id}/approve',
        data=json.dumps({}).encode('utf-8'),
        headers={'Content-Type': 'application/json', 'Authorization': f'Bearer {buyer_token}'}
    )
    with urllib.request.urlopen(req_buyer_approve) as resp:
        approve_res = json.loads(resp.read().decode())
        print(f"8. Buyer Approval: {approve_res['message']}, Status = {approve_res['status']}")

    # 9. Verification status endpoint
    req_verif = urllib.request.Request(
        f'{base_url}/documents/{doc_id}/verification',
        headers={'Authorization': f'Bearer {buyer_token}'}
    )
    with urllib.request.urlopen(req_verif) as resp:
        verif_res = json.loads(resp.read().decode())
        print(f"9. Verification status: Database status = {verif_res['document']['database_status']}")

    print("========================================")
    print("ALL TEST STEPS PASSED SUCCESSFULLY!")
    print("========================================")

if __name__ == '__main__':
    run_test()
