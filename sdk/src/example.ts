import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { Wallet, Idl } from "@coral-xyz/anchor";
import BN from "bn.js";
import { HyperboreBridgeSDK, IDL, BridgeEventListener } from "./index";
import { ethereumAddressToUint8Array, hexToUint8Array } from "./utils";

// This is just an example - you would need to fill in these values
const exampleUsage = async () => {
	// Setup connection and wallet
	const connection = new Connection("https://api.devnet.solana.com");
	const keypair = Keypair.generate(); // In real usage, load your actual keypair
	const wallet = new Wallet(keypair);
	const mutableIdl = JSON.parse(JSON.stringify(IDL)) as Idl;
	// Initialize SDK
	const bridgeSDK = new HyperboreBridgeSDK(
		connection,
		wallet,
		{
			programId: "qbuMdeYxYJXBjU6C6qFKjZKjXmrU83eDQomHdrch826", // Example program ID
			usdcMint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // Example USDC mint (devnet/mainnet)
		},
		mutableIdl
	);

	// Check pool state
	const poolState = await bridgeSDK.getPoolState();
	console.log("Pool paused:", poolState.paused);
	console.log("Tax basis points:", poolState.tax);

	// Example deposit
	try {
		const amount = new BN("1000000"); // 1 USDC (6 decimals)
		const evmAddress = ethereumAddressToUint8Array(
			"0x742d35Cc6634C0532925a3b844Bc454e4438f44e"
		);

		const depositTx = await bridgeSDK.depositUSDC(amount, evmAddress);
		console.log("Deposit transaction successful:", depositTx);
	} catch (error) {
		console.error("Deposit failed:", error);
	}

	// Example withdrawal with validator signatures
	try {
		const amount = new BN("1000000"); // 1 USDC
		const senderEvmAddress = ethereumAddressToUint8Array(
			"0x742d35Cc6634C0532925a3b844Bc454e4438f44e"
		);
		const recipient = new PublicKey(keypair.publicKey); // In real usage, this would be the recipient's address
		const nonce = hexToUint8Array(
			"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
		);

		// In a real app, these signatures would come from validators
		const signatures = {
			r: [
				hexToUint8Array(
					"1111111111111111111111111111111111111111111111111111111111111111"
				),
				hexToUint8Array(
					"2222222222222222222222222222222222222222222222222222222222222222"
				),
			],
			s: [
				hexToUint8Array(
					"3333333333333333333333333333333333333333333333333333333333333333"
				),
				hexToUint8Array(
					"4444444444444444444444444444444444444444444444444444444444444444"
				),
			],
			v: [27, 28],
		};

		const withdrawTx = await bridgeSDK.withdrawUSDC(
			amount,
			senderEvmAddress,
			recipient,
			nonce,
			signatures
		);
		console.log("Withdrawal transaction successful:", withdrawTx);
	} catch (error) {
		console.error("Withdrawal failed:", error);
	}

	// Get pool statistics
	const stats = await bridgeSDK.getPoolStatistics();
	console.log("Total volume:", stats.totalVolume.toString());
	console.log("Accumulated fees:", stats.accumulatedFees.toString());
};

// Don't actually run this - it's just an example
// exampleUsage().catch(console.error);
