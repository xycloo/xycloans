# Prototype specification

### Key concepts
- the actions should be executed by an apposite user-contract rather than split in actions to be executed.
- the checks should only be done by the Lender contract, if the receiver contract doesn't abide to the lending rules, the tx is reverted.
  - we're using try in our calls to catch the possible errors like not enough allowance to spend and return then with something like `ContractError(4)`

### Workflow

-> = invocation
=> = transfer

Preconditions: the interest $i$ is fixed.

(1) A -> Lender    :  A calls the lender with the amount to borrow $a$, the contract R to invoke (and the parameters).
(2) Lender => R    :  Lender transfers the amount to the receiver contract.
(3) Lender -> R    :  Lender invokes the Receiver contract's fn as specified by A in (1)
(4) R -> External  :  The receiving contract can now start executing the the yielding operations.
(5) Lender checks  :  The Lender now tries to try_xfer from the receiver to itself $a + i$:
					   	- if it succeeds, it means that the lend-yield-borrow operation worked.
						- if it doesn't, it means that the receiver contract either hasn't set up the correct allowance ($a$ + $i$, which is known since the interest is fixed).
