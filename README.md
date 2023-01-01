# XycLoans Protocol
### A flash loans protocol on Soroban

<hr/>

## Table of contents
1. [Links](#links)
2. [Technical Specification](#technical-specification)
    1. [Proxy Contract](#proxy-contract)
    2. [Vault Contract](#vault-contract)
    3. [Flash Loan Contract](#fl-contract)
    4. [Protocol](#protocol-contract)

### Links <a name="links"></a>
- [Home](https://floans.xycloo.com/)
- [Discord](#)

<hr/>

## Technical Specification <a name="technical-specification"></a>

<hr/>

### Proxy Contract <a name="proxy-contract"></a>

The proxy contract servers as gateway for users and developers to interact with the protocol, while maintaining great upgradeability standards for better security and metering over time.

Every invocation ideally passes though the proxy, which has three traits:
- Liquidity Provider trait:
    - providing liquidity
    - withdrawing fees
    - withdrawing liquidity position
- Borrower trait:
    - borrowing 
- Admin trait:
    - add and change vaults for a certain token.
    - add and change flash loans for a certain token.

<hr/>

### Vault Contract <a name="vault-contract"></a>

The vaults network is the vault structure behind the liquidity held in flash loans. Every token has its own vault, which holds fees the flash loans for that token produces and manages the liquidity that enters and exits the respective token's flash loan.

The vault has three key functionalities:
- Depositing. 
- Withdrawing fees.
- Withdrawing liquidity position.

#### Depositing

When a provider makes a deposit, the deposited amount ($a$) will go to the flash loan's balance. Additionally, the vault will mint fee shares to the provider with the following formula:

\[ s = \frac{a \cdot S}{B} \]

Where:
- $s$ is the amount of fee shares to mint.
- $S$ is the total circulating supply of the fee shares.
- $B$ is the total balance of the vault + total balance of the flash loan.

#### Fees
The flash loans generate over time a certain yield thorugh the borrowing interest (defined by governance). This yield is stored in the flash loan's vault and can be withdrawn by liquidity providers by burning their fee shares. 

Fee shares are minted as batches that are stored in the vault's storage. These batches keep track of the number of fee shares that are circulating within that batch. Every time that a provider withdraws their fee shares, they will be minted new ones in another batch.

As a result, a provider can withdraw only a portion of a batch's fees, get new shares minted and have two active fee batches that will hold different values.

When a provider withdraws a certain amount of fee shares $s$ the fees to send to the provider are calculated with:

\[ a_{fees} = \frac{B \cdot s}{S} - D \cdot \frac{s}{s_{initial}} \]

Where:
- $D$ is the batch's deposit amount.
- $s_{initial}$ is the amount of shares that were minted upon the batches creation.

After the calculated portion of calculated fees is sent to the provider, the vault mints new shares to the provider in a new batch. The minted amount of shares is calculated with:

\[ s = \frac{D \cdot s_{burned} \cdot S}{B \cdot s_{initial}- D \cdot s_{burned}} \]

#### Withdrawing the liquidity position

When the provider wishes to stop providing liquidity for the protocol, the vault offers a function to collect all the provider's fees (that might be sparse within different batches) and return the user's initial deposit as well.

The initial deposit is always kept in the flash loan, and there is no risk of it decreasing in value. **Providing liquidity in our protocol is always risk-free**.

However, as in every protocol, an attack might compromise your capital when it's stored in our protocol's vaults. However we take security very seriously and will undergo continuous private security assessments before deploying on the main network.

<hr/>

### Flash Loan Contract <a name="fl-contract"></a>

The flash loan contract has two main interfaces:
- lender interface
- borrower interface

The lender interface consists in a function to withdraw a certain amount of balance to a certain user, and can be called only by the flash loan's liquidity provider, which is the respective token's vault. The borrower interface allows to initiate the flash loan, calling a receiver contract after having funded it with the requested amount and lastly requiring the receiver contract to create an allowance to return the borrowed money plus an interest.

<hr/>

### Protocol <a name="protocol-contract"></a>
We haven't built the protocol contract with its governance yet.
