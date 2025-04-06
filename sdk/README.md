# Hyperbore Bridge SDK

This SDK provides a convenient way to interact with the Hyperbore Bridge Solana program. It simplifies the process of depositing and withdrawing USDC between Solana and EVM chains.

## Installation

```bash
npm install @hyperboredao/bridge-sdk
```

## Usage

### Initializing the SDK

```typescript
import { HyperboreBridgeSDK } from '@hyperboredao/bridge-sdk';
import { Connection, Keypair } from '@solana/web3.js';
import { Wallet } from '@coral-xyz/anchor';
import BN from 'bn.js';

// Import the IDL directly
// You'll need to export this from your anchor workspace
import { IDL } from './bridge_sol_idl';

// Create a connection to Solana
const connection = new Connection('https://api.mainnet-beta.solana.com');

// Set up a wallet
const keypair = Keypair.fromSecretKey(/* your secret key */);
const wallet = new Wallet(keypair);

// Initialize the SDK
const bridgeSDK = new HyperboreBridgeSDK(
  connection,
  wallet,
  {
    programId: 'qbuMdeYxYJXBjU6C6qFKjZKjXmrU83eDQomHdrch826', // The bridge program ID
    usdcMint: 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v', // USDC mint address
  },
  IDL
);
```

### Depositing USDC to Bridge to EVM

```typescript
import { ethereumAddressToUint8Array } from '@hyperboredao/bridge-sdk/utils';
import BN from 'bn.js';

// Create amount (in USDC base units, 6 decimals)
const amount = new BN('1000000'); // 1 USDC

// Recipient's Ethereum address
const ethAddress = '0x742d35Cc6634C0532925a3b844Bc454e4438f44e';
const recipientEvmAddress = ethereumAddressToUint8Array(ethAddress);

// Deposit USDC
const txHash = await bridgeSDK.depositUSDC(amount, recipientEvmAddress);
console.log('Deposit transaction:', txHash);
```

### Withdrawing USDC from the Bridge (after transferring from EVM)

```typescript
import { PublicKey } from '@solana/web3.js';
import { hexToUint8Array } from '@hyperboredao/bridge-sdk/utils';
import BN from 'bn.js';

// Amount to withdraw (in USDC base units, 6 decimals)
const amount = new BN('1000000'); // 1 USDC

// EVM sender address
const senderEvmAddress = ethereumAddressToUint8Array('0x742d35Cc6634C0532925a3b844Bc454e4438f44e');

// Solana recipient
const recipient = new PublicKey('GkXn6VfpUZUmnGwHBg8oKxpP4cJQnGMPc5UYQTKCodZ2');

// The nonce from the EVM transaction
const nonce = hexToUint8Array('0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef');

// Validator signatures (these would be collected from the validator network)
const signatures = {
  r: [
    hexToUint8Array('signature-r-component-1'),
    hexToUint8Array('signature-r-component-2'),
    // Add more r components as needed
  ],
  s: [
    hexToUint8Array('signature-s-component-1'),
    hexToUint8Array('signature-s-component-2'),
    // Add more s components as needed
  ],
  v: [
    27, // v component 1
    28, // v component 2
    // Add more v components as needed
  ],
};

// Withdraw USDC
const txHash = await bridgeSDK.withdrawUSDC(
  amount,
  senderEvmAddress,
  recipient,
  nonce,
  signatures
);
console.log('Withdrawal transaction:', txHash);
```

### Checking Pool State

```typescript
// Check if the bridge is paused
const isPaused = await bridgeSDK.isPoolPaused();
console.log('Bridge paused:', isPaused);

// Get pool statistics
const stats = await bridgeSDK.getPoolStatistics();
console.log('Total volume:', stats.totalVolume.toString());
console.log('Accumulated fees:', stats.accumulatedFees.toString());
console.log('Tax basis points:', stats.taxBasisPoints);

// Get active validators
const validators = await bridgeSDK.getValidators();
console.log('Active validators:', validators.map(v => v.toString()));

// Get required signatures
const requiredSigs = await bridgeSDK.getRequiredSignatures();
console.log('Required signatures:', requiredSigs);
```

## Message Signing Utilities

The SDK includes utilities for building and verifying message signatures in the same format used by the Solana program:

```typescript
import { buildMessage, ethereumAddressToUint8Array } from '@hyperboredao/bridge-sdk/utils';
import { PublicKey } from '@solana/web3.js';
import BN from 'bn.js';

// Build a message hash for signature verification
const nonce = hexToUint8Array('0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef');
const amount = new BN('1000000');
const evmAddress = ethereumAddressToUint8Array('0x742d35Cc6634C0532925a3b844Bc454e4438f44e');
const solanaAddress = new PublicKey('GkXn6VfpUZUmnGwHBg8oKxpP4cJQnGMPc5UYQTKCodZ2');

const messageHash = buildMessage(nonce, amount, evmAddress, solanaAddress);
console.log('Message hash:', Buffer.from(messageHash).toString('hex'));
```

## License

This project is licensed under the MIT License.