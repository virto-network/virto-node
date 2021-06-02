rpc_endpoint="http://localhost:9979" # change to your node endpoint
# Submit a new key via RPC, connect to where your `rpc-port` is listening
curl "$rpc_endpoint" -H "Content-Type:application/json;charset=utf-8" -d "@./aura_key.json"