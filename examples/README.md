# Examples

Collection of examples for borrowing a flash loan through xycLoans.

Currently:
- [simple](#how-to-use-simple)

## How to Use: [simple](./simple/)

[Simple](./simple/) is a basic initializable receiver contract that needs to be funded before executing the flash loan (as the contract does not perform any actions besides interacting with xycLoans' pools, thus isn't generating profit).

### Receiver logic

The receiver contract receives the borrowed amount and should perform a set of operations, such as arbitrage trades or liquidations for example, that end up being immediately profitable.

Currently the best way of trying out the receiver contract without implementing these kind of operations is to just fund it so that it can create allowances paying the interest with the funded amounts:

```
soroban contract invoke --id CB64D3G7SM2RTH6JSGG34DDTFTQ5CFDKVDZJZSODMCX4NJ2HV2KN7OHT --source-account $YOUR_SECRET --rpc-url https://rpc-futurenet.stellar.org/ --network-passphrase 'Test SDF Future Network ; October 2022' -- transfer --from $YOUR_PUBLIC_KEY --to $DEPLOYED_RECEIVER_CONTRACT --amount 100000000
```

To get started with using simple to request a 10 XLM flash loan, you can initialize it as follows:

```
soroban contract invoke --id $DEPLOYED_RECEIVER_ADDRESS --source-account $YOUR_SECRET --rpc-url https://rpc-futurenet.stellar.org/ --network-passphrase 'Test SDF Future Network ; October 2022' -- init --amount 100000000 --fl_address CCAQUB3T22JF7TSYTRKPJ4MN6EZK63MZZLN5H775Y5TU5S63MGYJB22Q --token_id CB64D3G7SM2RTH6JSGG34DDTFTQ5CFDKVDZJZSODMCX4NJ2HV2KN7OHT
```

### Borrowing the flash loan

Then borrow the flash loan from the XLM xycLoan pool:

```
soroban contract invoke --id CCAQUB3T22JF7TSYTRKPJ4MN6EZK63MZZLN5H775Y5TU5S63MGYJB22Q --source-account $YOUR_SECRET --rpc-url https://rpc-futurenet.stellar.org:443 --network-passphrase 'Test SDF Future Network ; October 2022' -- borrow --receiver_id $DEPLOYED_RECEIVER_ADDRESS --amount 100000000
```
