import { Connection, PublicKey } from "@solana/web3.js";
import {
	Program,
	BorshCoder,
	EventParser,
	AnchorProvider,
	Idl,
} from "@coral-xyz/anchor";
import { IDL } from "./idl";
import { DepositEvent, WithdrawEvent, BridgeEvent } from "./types";
import BN from "bn.js";

export class BridgeEventListener {
	private connection: Connection;
	private program: Program;
	private eventParser: EventParser;
	private programId: PublicKey;
	private provider: AnchorProvider;

	constructor(connection: Connection, programId: string, wallet: any) {
		this.connection = connection;
		this.programId = new PublicKey(programId);
		this.provider = new AnchorProvider(
			connection,
			wallet,
			AnchorProvider.defaultOptions()
		);
		// Create a dummy program instance just for parsing events
		this.program = new Program(IDL, this.provider);
		const mutableIdl = JSON.parse(JSON.stringify(IDL)) as Idl;
		this.eventParser = new EventParser(
			this.programId,
			new BorshCoder(mutableIdl)
		);
	}

	/**
	 * Listen for new deposit events
	 * @param callback Function to call when a deposit event is received
	 * @returns Subscription ID
	 */
	public subscribeToDepositEvents(
		callback: (event: DepositEvent) => void
	): number {
		// Subscribe to program account changes
		const subscriptionId = this.connection.onLogs(
			this.programId,
			(logs) => {
				if (!logs.logs || !logs.logs.length) return;

				try {
					// Parse events from logs
					const events = this.eventParser.parseLogs(logs.logs);

					// Find and process deposit events
					for (const event of events) {
						if (event.name === "USDCDeposited") {
							const data = event.data;
							const depositEvent: DepositEvent = {
								type: "deposit",
								data: {
									address: data.address.toString(),
									depositor: data.depositor.toString(),
									recipientEvmAddress:
										"0x" +
										Buffer.from(data.recipient_evm_address).toString("hex"),
									amount: new BN(data.amount.toString()),
									tax: new BN(data.tax.toString()),
									nonce: new BN(data.nonce.toString()),
									timestamp: data.timestamp,
								},
								signature: logs.signature,

								timestamp: Date.now(), // Add client-side timestamp
							};

							callback(depositEvent);
						}
					}
				} catch (error) {
					console.error("Error parsing deposit event:", error);
				}
			},
			"confirmed"
		);

		return subscriptionId;
	}

	/**
	 * Listen for new withdrawal events
	 * @param callback Function to call when a withdrawal event is received
	 * @returns Subscription ID
	 */
	public subscribeToWithdrawEvents(
		callback: (event: WithdrawEvent) => void
	): number {
		// Subscribe to program account changes
		const subscriptionId = this.connection.onLogs(
			this.programId,
			(logs) => {
				if (!logs.logs || !logs.logs.length) return;

				try {
					// Parse events from logs
					const events = this.eventParser.parseLogs(logs.logs);

					// Find and process withdrawal events
					for (const event of events) {
						if (event.name === "USDCWithdrawn") {
							const data = event.data;
							const withdrawEvent: WithdrawEvent = {
								type: "withdraw",
								data: {
									address: data.address.toString(),
									recipient: data.recipient.toString(),
									amount: new BN(data.amount.toString()),
									timestamp: data.timestamp,
								},
								signature: logs.signature,
								timestamp: Date.now(), // Add client-side timestamp
							};

							callback(withdrawEvent);
						}
					}
				} catch (error) {
					console.error("Error parsing withdraw event:", error);
				}
			},
			"confirmed"
		);

		return subscriptionId;
	}

	/**
	 * Listen for all bridge events
	 * @param callback Function to call when any bridge event is received
	 * @returns Subscription ID
	 */
	public subscribeToAllEvents(callback: (event: BridgeEvent) => void): number {
		// Subscribe to program account changes
		const subscriptionId = this.connection.onLogs(
			this.programId,
			(logs) => {
				if (!logs.logs || !logs.logs.length) return;

				try {
					// Parse events from logs
					const events = this.eventParser.parseLogs(logs.logs);

					// Process all events
					for (const event of events) {
						const bridgeEvent: BridgeEvent = {
							type: event.name,
							data: event.data,
							signature: logs.signature,
							timestamp: Date.now(),
						};

						callback(bridgeEvent);
					}
				} catch (error) {
					console.error("Error parsing bridge event:", error);
				}
			},
			"confirmed"
		);

		return subscriptionId;
	}

	/**
	 * Stop listening for events
	 * @param subscriptionId The subscription ID returned by the subscribe method
	 */
	public unsubscribe(subscriptionId: number): Promise<void> {
		return this.connection.removeOnLogsListener(subscriptionId);
	}

	/**
	 * Get deposit events from the past
	 * @param limit Maximum number of events to fetch
	 * @returns Array of deposit events
	 */
	public async getRecentDepositEvents(limit = 10): Promise<DepositEvent[]> {
		// Get recent signatures for program
		const signatures = await this.connection.getSignaturesForAddress(
			this.programId,
			{ limit }
		);

		const depositEvents: DepositEvent[] = [];

		// Process each transaction
		for (const signatureInfo of signatures) {
			try {
				const tx = await this.connection.getParsedTransaction(
					signatureInfo.signature,
					"confirmed"
				);

				if (!tx || !tx.meta || !tx.meta.logMessages) continue;

				// Parse events from logs
				const events = this.eventParser.parseLogs(tx.meta.logMessages);

				// Find deposit events
				for (const event of events) {
					if (event.name === "USDCDeposited") {
						const data = event.data;
						const depositEvent: DepositEvent = {
							type: "deposit",
							data: {
								address: data.address.toString(),
								depositor: data.depositor.toString(),
								recipientEvmAddress:
									"0x" +
									Buffer.from(data.recipient_evm_address).toString("hex"),
								amount: new BN(data.amount.toString()),
								tax: new BN(data.tax.toString()),
								nonce: new BN(data.nonce.toString()),
								timestamp: data.timestamp,
							},
							signature: signatureInfo.signature,
							timestamp: signatureInfo.blockTime
								? signatureInfo.blockTime * 1000
								: Date.now(),
						};

						depositEvents.push(depositEvent);
					}
				}
			} catch (error) {
				console.error("Error processing transaction:", error);
			}
		}

		return depositEvents;
	}
}

export default BridgeEventListener;
