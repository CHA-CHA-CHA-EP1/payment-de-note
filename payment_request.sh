curl -X POST http://127.0.0.1:8080/payment/initiate \
  -H "Content-Type: application/json" \
  -d '{
    "amount": 1500,
    "payment_method": "bank"
  }'
