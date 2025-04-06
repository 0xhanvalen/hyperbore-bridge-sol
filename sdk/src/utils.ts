import { PublicKey } from '@solana/web3.js';
import { createHash } from 'crypto';
import BN from 'bn.js';

/**
 * Builds a standardized message hash that matches the Solana program's build_message function.
 * This is used to verify signatures for cross-chain transactions.
 * 
 * @param nonce - 32-byte unique transaction identifier
 * @param amount - Amount to withdraw as BN
 * @param sourceAddress - 20-byte EVM address
 * @param destAddress - Solana public key
 * @returns 32-byte message hash
 */
export function buildMessage(
  nonce: Uint8Array,
  amount: BN,
  sourceAddress: Uint8Array,
  destAddress: PublicKey
): Uint8Array {
  // Ensure inputs are the correct size
  if (nonce.length !== 32) {
    throw new Error('Nonce must be 32 bytes');
  }
  
  if (sourceAddress.length !== 20) {
    throw new Error('Source address must be 20 bytes (EVM address)');
  }
  
  // Create buffer to match Solana program's implementation
  const message = Buffer.alloc(1 + 32 + 32 + 32 + 32); // Total 129 bytes
  
  // Current position in the buffer
  let position = 0;
  
  // Add chain identifier (1 byte)
  message[position] = 1;
  position += 1;
  
  // Add nonce (32 bytes)
  Buffer.from(nonce).copy(message, position);
  position += 32;
  
  // Add amount (convert to 32 bytes big-endian)
  const amountBytes = Buffer.alloc(32, 0);
  const amountBe = amount.toBuffer('be', 8); // Convert BN to 8-byte BE buffer
  amountBe.copy(amountBytes, 32 - amountBe.length); // Pad with leading zeros
  amountBytes.copy(message, position);
  position += 32;
  
  // Add source address (padded to 32 bytes)
  const sourceAddressPadded = Buffer.alloc(32, 0);
  Buffer.from(sourceAddress).copy(sourceAddressPadded, 32 - sourceAddress.length);
  sourceAddressPadded.copy(message, position);
  position += 32;
  
  // Add destination address (padded to 32 bytes)
  const destAddressPadded = Buffer.alloc(32, 0);
  Buffer.from(destAddress.toBuffer()).copy(destAddressPadded, 32 - destAddress.toBuffer().length);
  destAddressPadded.copy(message, position);
  
  // Hash the message using SHA-256 (matching Solana program)
  const hash = createHash('sha256').update(message).digest();
  
  return new Uint8Array(hash);
}

/**
 * Helper to convert hex string to Uint8Array
 */
export function hexToUint8Array(hex: string): Uint8Array {
  if (hex.startsWith('0x')) {
    hex = hex.substring(2);
  }
  
  if (hex.length % 2 !== 0) {
    throw new Error('Hex string must have an even number of characters');
  }
  
  const result = new Uint8Array(hex.length / 2);
  
  for (let i = 0; i < hex.length; i += 2) {
    result[i / 2] = parseInt(hex.substring(i, i + 2), 16);
  }
  
  return result;
}

/**
 * Helper to convert Uint8Array to hex string
 */
export function uint8ArrayToHex(bytes: Uint8Array): string {
  return Array.from(bytes)
    .map(b => b.toString(16).padStart(2, '0'))
    .join('');
}

/**
 * Convert an Ethereum address to Uint8Array
 */
export function ethereumAddressToUint8Array(address: string): Uint8Array {
  if (!address.startsWith('0x')) {
    throw new Error('Ethereum address must start with 0x');
  }
  
  if (address.length !== 42) { // 0x + 40 characters
    throw new Error('Ethereum address must be 42 characters (including 0x)');
  }
  
  return hexToUint8Array(address.substring(2));
}

/**
 * Calculate the expected tax amount for a transaction
 */
export function calculateTax(amount: BN, taxBasisPoints: number): BN {
  return amount.mul(new BN(taxBasisPoints)).div(new BN(10000));
}