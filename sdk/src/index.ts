import {
	Program,
	AnchorProvider,
	Idl,
	setProvider,
	web3,
} from "@coral-xyz/anchor";

// Export types and utilities
export * from "./types";
export * from "./utils";
export { IDL } from "./idl";
export { BridgeEventListener } from "./eventListener";
import { Connection, PublicKey, Keypair, Transaction } from "@solana/web3.js";
import {
	TOKEN_PROGRAM_ID,
	ASSOCIATED_TOKEN_PROGRAM_ID,
	getAssociatedTokenAddress,
	createAssociatedTokenAccountInstruction,
} from "@solana/spl-token";
import BN from "bn.js";

type PoolState = {
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
};

interface HyperboreBridgeSDKProgram extends Program {
	account: {
		poolState: {
			fetch: (address: PublicKey) => Promise<PoolState>;
		};
	};
}

// Import the IDL directly
// Note: We'll need to generate or acquire the IDL JSON
// import { IDL } from './idl/bridge_sol';

export interface BridgeConfig {
	programId: string;
	usdcMint: string;
}

export class HyperboreBridgeSDK {
	private connection: Connection;
	private provider: AnchorProvider;
	private program: Program;
	private programId: PublicKey;
	private usdcMint: PublicKey;
	private poolStateAddress: PublicKey;
	private poolStateBump: number;

	constructor(
		connection: Connection,
		wallet: any,
		config: BridgeConfig,
		idl: Idl
	) {
		this.connection = connection;
		this.provider = new AnchorProvider(
			connection,
			wallet,
			AnchorProvider.defaultOptions()
		);
		setProvider(this.provider);

		this.programId = new PublicKey(config.programId);
		this.usdcMint = new PublicKey(config.usdcMint);
		this.program = new Program(idl, this.provider) as HyperboreBridgeSDKProgram;

		// Derive the pool state PDA
		const [poolStateAddress, poolStateBump] = PublicKey.findProgramAddressSync(
			[Buffer.from("pool_state")],
			this.programId
		);

		this.poolStateAddress = poolStateAddress;
		this.poolStateBump = poolStateBump;
	}

	/**
	 * Get the bridge pool state data
	 */
	async getPoolState() {
		return await (
			this.program as HyperboreBridgeSDKProgram
		).account.poolState.fetch(this.poolStateAddress);
	}

	/**
	 * Checks if the pool is paused
	 */
	async isPoolPaused(): Promise<boolean> {
		const poolState = await this.getPoolState();
		return poolState.paused;
	}

	/**
	 * Calculate the tax amount for a deposit or withdrawal
	 */
	async calculateTax(amount: BN): Promise<BN> {
		const poolState = await this.getPoolState();
		const taxBasisPoints = poolState.tax;

		// Calculate tax: amount * taxBasisPoints / 10000
		return amount.mul(new BN(taxBasisPoints)).div(new BN(10000));
	}

	/**
	 * Deposit USDC to bridge to EVM
	 */
	async depositUSDC(
		amount: BN,
		recipientEvmAddress: Uint8Array, // 20-byte EVM address
		payer = this.provider.wallet.publicKey
	): Promise<string> {
		// Ensure recipient EVM address is 20 bytes
		if (recipientEvmAddress.length !== 20) {
			throw new Error("Recipient EVM address must be 20 bytes");
		}

		// Convert to the expected format
		const evmAddressArray = Array.from(recipientEvmAddress);

		// Get the user's USDC ATA
		const userUsdcAta = await getAssociatedTokenAddress(
			this.usdcMint,
			payer,
			false
		);

		// Get the pool's USDC ATA
		const poolUsdcAta = await getAssociatedTokenAddress(
			this.usdcMint,
			this.poolStateAddress,
			true
		);

		// Check if pool ATA exists, if not we'll create it
		const poolAtaInfo = await this.connection.getAccountInfo(poolUsdcAta);

		const tx = new Transaction();

		// If pool ATA doesn't exist, add instruction to create it
		if (!poolAtaInfo) {
			tx.add(
				createAssociatedTokenAccountInstruction(
					payer,
					poolUsdcAta,
					this.poolStateAddress,
					this.usdcMint
				)
			);
		}

		// Add the deposit instruction
		tx.add(
			await this.program.methods
				.depositUsdc({
					amount: amount,
					recipientEvmAddress: evmAddressArray,
				})
				.accounts({
					depositor: payer,
					poolState: this.poolStateAddress,
					mintAccount: this.usdcMint,
					depositorAta: userUsdcAta,
					poolAta: poolUsdcAta,
					tokenProgram: TOKEN_PROGRAM_ID,
					associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
					systemProgram: web3.SystemProgram.programId,
				})
				.instruction()
		);

		// Send and confirm transaction
		const signature = await this.provider.sendAndConfirm(tx);
		return signature;
	}

	/**
	 * Withdraw USDC from bridge (after transferring from EVM)
	 */
	async withdrawUSDC(
		amount: BN,
		senderEvmAddress: Uint8Array, // 20-byte EVM address
		recipient: PublicKey,
		nonce: Uint8Array, // 32-byte nonce
		signatures: {
			r: Uint8Array[];
			s: Uint8Array[];
			v: number[];
		},
		payer = this.provider.wallet.publicKey
	): Promise<string> {
		// Validate inputs
		if (senderEvmAddress.length !== 20) {
			throw new Error("Sender EVM address must be 20 bytes");
		}

		if (nonce.length !== 32) {
			throw new Error("Nonce must be 32 bytes");
		}

		// Get recipient's USDC ATA
		const recipientUsdcAta = await getAssociatedTokenAddress(
			this.usdcMint,
			recipient,
			false
		);

		// Get the pool's USDC ATA
		const poolUsdcAta = await getAssociatedTokenAddress(
			this.usdcMint,
			this.poolStateAddress,
			true
		);

		// Check if recipient ATA exists, if not we'll create it
		const recipientAtaInfo = await this.connection.getAccountInfo(
			recipientUsdcAta
		);

		const tx = new Transaction();

		// If recipient ATA doesn't exist, add instruction to create it
		if (!recipientAtaInfo) {
			tx.add(
				createAssociatedTokenAccountInstruction(
					payer,
					recipientUsdcAta,
					recipient,
					this.usdcMint
				)
			);
		}

		// Process signatures into expected format
		const MAX_VALIDATORS = 16;

		// Ensure arrays are of the right size
		const paddedR = [...signatures.r];
		const paddedS = [...signatures.s];
		const paddedV = [...signatures.v];

		// Pad arrays to MAX_VALIDATORS length with empty values
		while (paddedR.length < MAX_VALIDATORS) {
			paddedR.push(new Uint8Array(32));
		}

		while (paddedS.length < MAX_VALIDATORS) {
			paddedS.push(new Uint8Array(32));
		}

		while (paddedV.length < MAX_VALIDATORS) {
			paddedV.push(0);
		}

		// Add the withdraw instruction
		tx.add(
			await this.program.methods
				.withdrawUsdc(recipient, {
					amount: amount,
					senderEvmAddress: Array.from(senderEvmAddress),
					nonce: Array.from(nonce),
					r: paddedR.map((r) => Array.from(r)),
					s: paddedS.map((s) => Array.from(s)),
					v: paddedV,
				})
				.accounts({
					payer: payer,
					poolState: this.poolStateAddress,
					mintAccount: this.usdcMint,
					recipientAta: recipientUsdcAta,
					poolAta: poolUsdcAta,
					tokenProgram: TOKEN_PROGRAM_ID,
					associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
					systemProgram: web3.SystemProgram.programId,
				})
				.instruction()
		);

		// Send and confirm transaction
		const signature = await this.provider.sendAndConfirm(tx);
		return signature;
	}

	/**
	 * Get total pool volume statistics
	 */
	async getPoolStatistics() {
		const poolState = await this.getPoolState();
		return {
			totalVolume: poolState.totalVolume,
			accumulatedFees: poolState.accumulatedFees,
			taxBasisPoints: poolState.tax,
		};
	}

	/**
	 * Get list of active validators
	 */
	async getValidators(): Promise<PublicKey[]> {
		const poolState = await this.getPoolState();
		const validators = poolState.validators;
		// Filter out the default pubkey (all zeros)
		return validators.filter(
			(validator) => !validator.equals(PublicKey.default)
		);
	}

	/**
	 * Get required number of signatures for withdrawals
	 */
	async getRequiredSignatures(): Promise<number> {
		const poolState = await this.getPoolState();
		return poolState.requiredSignatures;
	}
}

export default HyperboreBridgeSDK;
