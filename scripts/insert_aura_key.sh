rpc_endpoint="http://localhost:9979" # change to your node endpoint
# Submit a new key via RPC, connect to where your `rpc-port` is listening
# the @aura_key.json file must be in the following format
# {
#     "jsonrpc":"2.0",
#     "id":1,
#     "method":"author_insertKey",
#     "params": [
#       "aura",
#       "<mnemonic phrase>",
#       "<public key>"
#     ]
# }
curl "$rpc_endpoint" -H "Content-Type:application/json;charset=utf-8" -d "@./aura_key.json"

