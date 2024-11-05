#!/bin/bash

#set -x

function curl_fn {
  curl -q -H "Accept: application/json" -H "Content-Type: application/json" "$@"
}

echo "network/status"
curl_fn "http://localhost:3337/mesh/network/status" \
  -d '{"blockchain": "radix", "network": "localnet"}' | jq '.'

echo "network/list"
curl_fn "http://localhost:3337/mesh/network/list" \
  -d '{}' | jq '.'

echo "network/options"
curl_fn "http://localhost:3337/mesh/network/options" \
  -d '{"blockchain": "radix", "network": "localnet"}' | jq '.'

echo "account/balance"
# curl_fn "http://localhost:3337/mesh/account/balance" \
#   -d '{"network_identifier": {"blockchain": "radix", "network": "localnet"}, "account_identifier": {"address" : "account_loc1cxx8q2ttn8kh9gy34lp98ek9a22jd7ljdsna6m78nh7esl6mfl7ku6" }, "currencies": [ { "symbol": "resource_loc1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxvq32hv", "decimals": 18 }, { "symbol": "resource_loc1tkwk3jpp92j7j0acpnxh82xwdz3dd6n57tt46fn9sntud57junt0dv", "decimals": 0 } ]}' | jq '.'

curl_fn "http://localhost:3337/mesh/account/balance" \
  -d '{"network_identifier": {"blockchain": "radix", "network": "localnet"}, "account_identifier": {"address" : "account_loc1c92tk2ndakxvess83jq4kal7dpxnda05a39sl5hz0926tls04v648d" }, "currencies": [ { "symbol": "resource_loc1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxvq32hv", "decimals": 18 }, { "symbol": "resource_loc1tkwk3jpp92j7j0acpnxh82xwdz3dd6n57tt46fn9sntud57junt0dv", "decimals": 0 }, { "symbol": "resource_loc1t5re6ahdn7z2tsczgduct3xzuerpxujqtvukwmxvgs2z3rcyh6agjg", "decimals": 18 } ]}' | jq '.'

curl_fn "http://localhost:3337/mesh/account/balance" \
  -d '{"network_identifier": {"blockchain": "radix", "network": "localnet"}, "block_identifier": {"index": 1000 },  "account_identifier": {"address" : "account_loc1c92tk2ndakxvess83jq4kal7dpxnda05a39sl5hz0926tls04v648d" }, "currencies": [ { "symbol": "resource_loc1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxvq32hv", "decimals": 18 }, { "symbol": "resource_loc1tkwk3jpp92j7j0acpnxh82xwdz3dd6n57tt46fn9sntud57junt0dv", "decimals": 0 } ]}' | jq '.'

#| jq '.'
# {"network_identifier": {"blockchain": "radix", "network": "localnet"}, \
#        "account_identifier": {"address" : "account_loc1cxx8q2ttn8kh9gy34lp98ek9a22jd7ljdsna6m78nh7esl6mfl7ku6" }, \
#        "currencies": [ \
#           { "symbol": "resource_loc1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxvq32hv", "decimals": 18 }, \
#           { "symbol": "resource_loc1tkwk3jpp92j7j0acpnxh82xwdz3dd6n57tt46fn9sntud57junt0dv", "decimals": 18 } \
#       ]}
# # {
#     "network_identifier": {
#         "blockchain": "bitcoin",
#         "network": "mainnet",
#         "sub_network_identifier": {
#             "network": "shard 1",
#             "metadata": {
#                 "producer": "0x52bc44d5378309ee2abf1539bf71de1b7d7be3b5"
#             }
#         }
#     },
#     "account_identifier": {
#         "address": "0x3a065000ab4183c6bf581dc1e55a605455fc6d61",
#         "sub_account": {
#             "address": "0x6b175474e89094c44da98b954eedeac495271d0f",
#             "metadata": {}
#         },
#         "metadata": {}
#     },
#     "block_identifier": {
#         "index": 1123941,
#         "hash": "0x1f2cc6c5027d2f201a5453ad1119574d2aed23a392654742ac3c78783c071f85"
#     },
#     "currencies": [
#         {
#             "symbol": "BTC",
#             "decimals": 8,
#             "metadata": {
#                 "Issuer": "Satoshi"
#             }
#         }
#     ]
# }
