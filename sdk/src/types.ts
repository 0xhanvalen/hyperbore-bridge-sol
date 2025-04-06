import { PublicKey } from "@solana/web3.js";
import BN from "bn.js";

export interface PoolState {
	owner: PublicKey;
	proposedOwner: PublicKey;
	usdcMint: PublicKey;
	paused: boolean;
	validators: PublicKey[];
	requiredSignatures: number;
	tax: number;
	totalVolume: BN;
	accumulatedFees: BN;
	treasury: PublicKey;
	bump: number;
}

export interface DepositUSDCArgs {
	amount: BN;
	recipientEvmAddress: number[]; // 20-byte array
}

export interface WithdrawUSDCArgs {
	amount: BN;
	senderEvmAddress: number[]; // 20-byte array
	nonce: number[]; // 32-byte array
	r: number[][]; // Array of 32-byte arrays
	s: number[][]; // Array of 32-byte arrays
	v: number[]; // Array of numbers
}

export interface SignatureComponents {
	r: Uint8Array[];
	s: Uint8Array[];
	v: number[];
}

export interface BridgeEvent {
	type: string;
	data: any;
	signature: string;
	timestamp: number;
}

export interface DepositEvent extends BridgeEvent {
	type: "deposit";
	data: {
		address: string;
		depositor: string;
		recipientEvmAddress: string;
		amount: BN;
		tax: BN;
		nonce: BN;
		timestamp: number;
	};
}

export interface WithdrawEvent extends BridgeEvent {
	type: "withdraw";
	data: {
		address: string;
		recipient: string;
		amount: BN;
		timestamp: number;
	};
}

export interface PoolStatistics {
	totalVolume: BN;
	accumulatedFees: BN;
	taxBasisPoints: number;
}
