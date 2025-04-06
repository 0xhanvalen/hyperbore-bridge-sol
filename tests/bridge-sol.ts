import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BridgeSol } from "../target/types/bridge_sol";
import {
	TOKEN_PROGRAM_ID,
	createInitializeMintInstruction,
} from "@solana/spl-token";

describe("bridge-sol", () => {
	// Configure the client to use the local cluster.
	anchor.setProvider(anchor.AnchorProvider.env());

	const program = anchor.workspace.bridgeSol as Program<BridgeSol>;
	const user = anchor.web3.Keypair.generate();
	const treasury = anchor.web3.Keypair.generate();
	const usdcMint = anchor.web3.Keypair.generate();

	before(async () => {
		// Create the USDC Mint account
		const lamports =
			await program.provider.connection.getMinimumBalanceForRentExemption(
				82 // Mint account size is 82 bytes
			);

		const transaction = new anchor.web3.Transaction().add(
			anchor.web3.SystemProgram.createAccount({
				fromPubkey: program.provider.wallet.publicKey,
				newAccountPubkey: usdcMint.publicKey,
				lamports,
				space: 82,
				programId: TOKEN_PROGRAM_ID,
			}),
			createInitializeMintInstruction(
				usdcMint.publicKey, // Mint account public key
				6, // Decimals
				program.provider.wallet.publicKey, // Mint authority
				null, // Freeze authority (optional)
				TOKEN_PROGRAM_ID // Token program ID
			)
		);

		// Send and confirm the transaction
		await program.provider.sendAndConfirm(transaction, [usdcMint]); // Add `usdcMint` as a signer
	});

	it("Is initialized!", async () => {
		// Add your test here.
		// Create a treasury account

		// Fund the treasury account with some lamports
		const txFund = await program.provider.connection.requestAirdrop(
			user.publicKey,
			anchor.web3.LAMPORTS_PER_SOL
		);
		await program.provider.connection.confirmTransaction(txFund);

		// Initialize the program with the treasury account
		const tx = await program.methods
			.initialize(50)
			.accounts({
				usdcMint: usdcMint.publicKey,
				treasury: treasury.publicKey,
			})
			.signers([])
			.rpc();
		console.log("Your transaction signature", tx);
	});
});
