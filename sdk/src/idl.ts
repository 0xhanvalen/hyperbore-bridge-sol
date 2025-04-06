export const IDL = {
	address: "qbuMdeYxYJXBjU6C6qFKjZKjXmrU83eDQomHdrch826",
	metadata: {
		name: "bridge_sol",
		version: "0.1.0",
		spec: "0.1.0",
		description: "Created with Anchor",
	},
	instructions: [
		{
			name: "accept_ownership",
			discriminator: [172, 23, 43, 13, 238, 213, 85, 150],
			accounts: [
				{
					name: "signer",
					writable: true,
					signer: true,
				},
				{
					name: "pool_state",
					writable: true,
					pda: {
						seeds: [
							{
								kind: "const",
								value: [112, 111, 111, 108, 95, 115, 116, 97, 116, 101],
							},
						],
					},
				},
				{
					name: "system_program",
					address: "11111111111111111111111111111111",
				},
			],
			args: [],
		},
		{
			name: "add_validator",
			discriminator: [250, 113, 53, 54, 141, 117, 215, 185],
			accounts: [
				{
					name: "owner",
					writable: true,
					signer: true,
				},
				{
					name: "pool_state",
					writable: true,
					pda: {
						seeds: [
							{
								kind: "const",
								value: [112, 111, 111, 108, 95, 115, 116, 97, 116, 101],
							},
						],
					},
				},
				{
					name: "new_validator",
				},
				{
					name: "system_program",
					address: "11111111111111111111111111111111",
				},
			],
			args: [],
		},
		{
			name: "deposit_usdc",
			discriminator: [184, 148, 250, 169, 224, 213, 34, 126],
			accounts: [
				{
					name: "depositor",
					writable: true,
					signer: true,
				},
				{
					name: "pool_state",
					writable: true,
					pda: {
						seeds: [
							{
								kind: "const",
								value: [112, 111, 111, 108, 95, 115, 116, 97, 116, 101],
							},
						],
					},
				},
				{
					name: "mint_account",
					writable: true,
				},
				{
					name: "depositor_ata",
					writable: true,
				},
				{
					name: "pool_ata",
					writable: true,
					pda: {
						seeds: [
							{
								kind: "account",
								path: "pool_state",
							},
							{
								kind: "const",
								value: [
									6, 221, 246, 225, 215, 101, 161, 147, 217, 203, 225, 70, 206,
									235, 121, 172, 28, 180, 133, 237, 95, 91, 55, 145, 58, 140,
									245, 133, 126, 255, 0, 169,
								],
							},
							{
								kind: "account",
								path: "mint_account",
							},
						],
						program: {
							kind: "const",
							value: [
								140, 151, 37, 143, 78, 36, 137, 241, 187, 61, 16, 41, 20, 142,
								13, 131, 11, 90, 19, 153, 218, 255, 16, 132, 4, 142, 123, 216,
								219, 233, 248, 89,
							],
						},
					},
				},
				{
					name: "token_program",
					address: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
				},
				{
					name: "associated_token_program",
					address: "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
				},
				{
					name: "system_program",
					address: "11111111111111111111111111111111",
				},
			],
			args: [
				{
					name: "args",
					type: {
						defined: {
							name: "DepositUSDCArgs",
						},
					},
				},
			],
		},
		{
			name: "initialize",
			discriminator: [175, 175, 109, 31, 13, 152, 155, 237],
			accounts: [
				{
					name: "owner",
					writable: true,
					signer: true,
				},
				{
					name: "pool_state",
					writable: true,
					pda: {
						seeds: [
							{
								kind: "const",
								value: [112, 111, 111, 108, 95, 115, 116, 97, 116, 101],
							},
						],
					},
				},
				{
					name: "treasury",
				},
				{
					name: "usdc_mint",
				},
				{
					name: "system_program",
					address: "11111111111111111111111111111111",
				},
			],
			args: [
				{
					name: "tax",
					type: "u16",
				},
			],
		},
		{
			name: "remove_validator",
			discriminator: [25, 96, 211, 155, 161, 14, 168, 188],
			accounts: [
				{
					name: "owner",
					writable: true,
					signer: true,
				},
				{
					name: "pool_state",
					writable: true,
					pda: {
						seeds: [
							{
								kind: "const",
								value: [112, 111, 111, 108, 95, 115, 116, 97, 116, 101],
							},
						],
					},
				},
				{
					name: "old_validator",
				},
				{
					name: "system_program",
					address: "11111111111111111111111111111111",
				},
			],
			args: [],
		},
		{
			name: "update_state",
			discriminator: [135, 112, 215, 75, 247, 185, 53, 176],
			accounts: [
				{
					name: "owner",
					writable: true,
					signer: true,
				},
				{
					name: "pool_state",
					writable: true,
					pda: {
						seeds: [
							{
								kind: "const",
								value: [112, 111, 111, 108, 95, 115, 116, 97, 116, 101],
							},
						],
					},
				},
				{
					name: "system_program",
					address: "11111111111111111111111111111111",
				},
			],
			args: [
				{
					name: "args",
					type: {
						defined: {
							name: "ConfigUpdateArgs",
						},
					},
				},
			],
		},
		{
			name: "withdraw_fees",
			discriminator: [198, 212, 171, 109, 144, 215, 174, 89],
			accounts: [
				{
					name: "owner",
					writable: true,
					signer: true,
				},
				{
					name: "pool_state",
					writable: true,
					pda: {
						seeds: [
							{
								kind: "const",
								value: [112, 111, 111, 108, 95, 115, 116, 97, 116, 101],
							},
						],
					},
				},
				{
					name: "treasury",
				},
				{
					name: "mint_account",
					writable: true,
				},
				{
					name: "treasury_ata",
					writable: true,
				},
				{
					name: "pool_ata",
					pda: {
						seeds: [
							{
								kind: "account",
								path: "pool_state",
							},
							{
								kind: "const",
								value: [
									6, 221, 246, 225, 215, 101, 161, 147, 217, 203, 225, 70, 206,
									235, 121, 172, 28, 180, 133, 237, 95, 91, 55, 145, 58, 140,
									245, 133, 126, 255, 0, 169,
								],
							},
							{
								kind: "account",
								path: "mint_account",
							},
						],
						program: {
							kind: "const",
							value: [
								140, 151, 37, 143, 78, 36, 137, 241, 187, 61, 16, 41, 20, 142,
								13, 131, 11, 90, 19, 153, 218, 255, 16, 132, 4, 142, 123, 216,
								219, 233, 248, 89,
							],
						},
					},
				},
				{
					name: "token_program",
					address: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
				},
				{
					name: "associated_token_program",
					address: "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
				},
				{
					name: "system_program",
					address: "11111111111111111111111111111111",
				},
			],
			args: [],
		},
		{
			name: "withdraw_usdc",
			discriminator: [114, 49, 72, 184, 27, 156, 243, 155],
			accounts: [
				{
					name: "payer",
					writable: true,
					signer: true,
				},
				{
					name: "pool_state",
					writable: true,
					pda: {
						seeds: [
							{
								kind: "const",
								value: [112, 111, 111, 108, 95, 115, 116, 97, 116, 101],
							},
						],
					},
				},
				{
					name: "mint_account",
					writable: true,
				},
				{
					name: "recipient_ata",
					writable: true,
				},
				{
					name: "pool_ata",
					pda: {
						seeds: [
							{
								kind: "account",
								path: "pool_state",
							},
							{
								kind: "const",
								value: [
									6, 221, 246, 225, 215, 101, 161, 147, 217, 203, 225, 70, 206,
									235, 121, 172, 28, 180, 133, 237, 95, 91, 55, 145, 58, 140,
									245, 133, 126, 255, 0, 169,
								],
							},
							{
								kind: "account",
								path: "mint_account",
							},
						],
						program: {
							kind: "const",
							value: [
								140, 151, 37, 143, 78, 36, 137, 241, 187, 61, 16, 41, 20, 142,
								13, 131, 11, 90, 19, 153, 218, 255, 16, 132, 4, 142, 123, 216,
								219, 233, 248, 89,
							],
						},
					},
				},
				{
					name: "token_program",
					address: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
				},
				{
					name: "associated_token_program",
					address: "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
				},
				{
					name: "system_program",
					address: "11111111111111111111111111111111",
				},
			],
			args: [
				{
					name: "recipient",
					type: "pubkey",
				},
				{
					name: "args",
					type: {
						defined: {
							name: "WithdrawUSDCArgs",
						},
					},
				},
			],
		},
	],
	accounts: [
		{
			name: "PoolState",
			discriminator: [247, 237, 227, 245, 215, 195, 222, 70],
		},
	],
	events: [
		{
			name: "FeesWithdrawn",
			discriminator: [234, 15, 0, 119, 148, 241, 40, 21],
		},
		{
			name: "OwnerChanged",
			discriminator: [34, 223, 103, 225, 239, 231, 51, 53],
		},
		{
			name: "PoolCreated",
			discriminator: [202, 44, 41, 88, 104, 220, 157, 82],
		},
		{
			name: "PoolStateUpdated",
			discriminator: [231, 22, 226, 177, 26, 215, 227, 97],
		},
		{
			name: "USDCDeposited",
			discriminator: [75, 200, 113, 3, 12, 197, 106, 215],
		},
		{
			name: "USDCWithdrawn",
			discriminator: [58, 59, 209, 122, 222, 203, 160, 217],
		},
		{
			name: "ValidatorAdded",
			discriminator: [67, 26, 43, 25, 58, 219, 99, 48],
		},
		{
			name: "ValidatorRemoved",
			discriminator: [133, 140, 80, 83, 7, 209, 70, 130],
		},
	],
	errors: [
		{
			code: 6000,
			name: "TooManyValidators",
			msg: "Too many validators. Max is 10. Remove one before adding another one.",
		},
		{
			code: 6001,
			name: "ValidatorDoesNotExist",
			msg: "This validator key doesn't exist.",
		},
		{
			code: 6002,
			name: "InvalidOwnershipChange",
			msg: "Invalid Ownership Change, you are not the proposed owner.",
		},
		{
			code: 6003,
			name: "PoolPaused",
			msg: "Pool's Closed.",
		},
		{
			code: 6004,
			name: "WrongToken",
			msg: "USDC Only.",
		},
		{
			code: 6005,
			name: "ZeroTax",
			msg: "Tax must be at least 1 basis point.",
		},
		{
			code: 6006,
			name: "TaxFailed",
			msg: "Tax could not be applied.",
		},
		{
			code: 6007,
			name: "InvalidMessageHash",
			msg: "Verification failed - failed to hash message.",
		},
		{
			code: 6008,
			name: "InvalidSignature",
			msg: "Verification failed - one tested signature failed.",
		},
		{
			code: 6009,
			name: "InvalidPublicKey",
			msg: "Verification failed - one tested validator pubkey failed.",
		},
		{
			code: 6010,
			name: "NotEnoughSignatures",
			msg: "Verification failed - not enough signatures present.",
		},
		{
			code: 6011,
			name: "MismatchedSignaturesAndKeys",
			msg: "Verification failed - wrong number of sigs/validators.",
		},
		{
			code: 6012,
			name: "DuplicateValidator",
			msg: "Verification failed - attempted to pass duplicate validators.",
		},
		{
			code: 6013,
			name: "InvalidValidatorAccount",
			msg: "Invalid validator account supplied",
		},
		{
			code: 6014,
			name: "FailedToValidate",
			msg: "Failed to validate withdrawal",
		},
	],
	types: [
		{
			name: "ConfigUpdateArgs",
			type: {
				kind: "struct",
				fields: [
					{
						name: "treasury",
						type: {
							option: "pubkey",
						},
					},
					{
						name: "owner",
						type: {
							option: "pubkey",
						},
					},
					{
						name: "tax",
						type: {
							option: "u16",
						},
					},
					{
						name: "paused",
						type: {
							option: "bool",
						},
					},
				],
			},
		},
		{
			name: "DepositUSDCArgs",
			type: {
				kind: "struct",
				fields: [
					{
						name: "amount",
						type: "u64",
					},
					{
						name: "recipient_evm_address",
						type: {
							array: ["u8", 20],
						},
					},
				],
			},
		},
		{
			name: "FeesWithdrawn",
			type: {
				kind: "struct",
				fields: [
					{
						name: "address",
						type: "pubkey",
					},
					{
						name: "recipient",
						type: "pubkey",
					},
					{
						name: "amount",
						type: "u64",
					},
					{
						name: "timestamp",
						type: "i64",
					},
				],
			},
		},
		{
			name: "OwnerChanged",
			type: {
				kind: "struct",
				fields: [
					{
						name: "address",
						type: "pubkey",
					},
					{
						name: "new_owner",
						type: "pubkey",
					},
					{
						name: "timestamp",
						type: "i64",
					},
				],
			},
		},
		{
			name: "PoolCreated",
			type: {
				kind: "struct",
				fields: [
					{
						name: "address",
						type: "pubkey",
					},
					{
						name: "treasury",
						type: "pubkey",
					},
					{
						name: "tax",
						type: "u16",
					},
					{
						name: "timestamp",
						type: "i64",
					},
				],
			},
		},
		{
			name: "PoolState",
			type: {
				kind: "struct",
				fields: [
					{
						name: "owner",
						type: "pubkey",
					},
					{
						name: "proposed_owner",
						type: "pubkey",
					},
					{
						name: "usdc_mint",
						type: "pubkey",
					},
					{
						name: "paused",
						type: "bool",
					},
					{
						name: "validators",
						type: {
							array: ["pubkey", 16],
						},
					},
					{
						name: "required_signatures",
						type: "u8",
					},
					{
						name: "tax",
						type: "u16",
					},
					{
						name: "total_volume",
						type: "u64",
					},
					{
						name: "accumulated_fees",
						type: "u64",
					},
					{
						name: "treasury",
						type: "pubkey",
					},
					{
						name: "bump",
						type: "u8",
					},
				],
			},
		},
		{
			name: "PoolStateUpdated",
			type: {
				kind: "struct",
				fields: [
					{
						name: "address",
						type: "pubkey",
					},
					{
						name: "proposed_owner",
						type: "pubkey",
					},
					{
						name: "treasury",
						type: "pubkey",
					},
					{
						name: "tax",
						type: "u16",
					},
					{
						name: "paused",
						type: "bool",
					},
					{
						name: "timestamp",
						type: "i64",
					},
				],
			},
		},
		{
			name: "USDCDeposited",
			type: {
				kind: "struct",
				fields: [
					{
						name: "address",
						type: "pubkey",
					},
					{
						name: "depositor",
						type: "pubkey",
					},
					{
						name: "recipient_evm_address",
						type: {
							array: ["u8", 20],
						},
					},
					{
						name: "amount",
						type: "u64",
					},
					{
						name: "tax",
						type: "u64",
					},
					{
						name: "nonce",
						type: "u64",
					},
					{
						name: "timestamp",
						type: "i64",
					},
				],
			},
		},
		{
			name: "USDCWithdrawn",
			type: {
				kind: "struct",
				fields: [
					{
						name: "address",
						type: "pubkey",
					},
					{
						name: "recipient",
						type: "pubkey",
					},
					{
						name: "amount",
						type: "u64",
					},
					{
						name: "timestamp",
						type: "i64",
					},
				],
			},
		},
		{
			name: "ValidatorAdded",
			type: {
				kind: "struct",
				fields: [
					{
						name: "pool_state",
						type: "pubkey",
					},
					{
						name: "address",
						type: "pubkey",
					},
					{
						name: "required_validators",
						type: "u8",
					},
					{
						name: "timestamp",
						type: "i64",
					},
				],
			},
		},
		{
			name: "ValidatorRemoved",
			type: {
				kind: "struct",
				fields: [
					{
						name: "pool_state",
						type: "pubkey",
					},
					{
						name: "address",
						type: "pubkey",
					},
					{
						name: "required_validators",
						type: "u8",
					},
					{
						name: "timestamp",
						type: "i64",
					},
				],
			},
		},
		{
			name: "WithdrawUSDCArgs",
			type: {
				kind: "struct",
				fields: [
					{
						name: "amount",
						type: "u64",
					},
					{
						name: "sender_evm_address",
						type: {
							array: ["u8", 20],
						},
					},
					{
						name: "nonce",
						type: {
							array: ["u8", 32],
						},
					},
					{
						name: "r",
						type: {
							array: [
								{
									array: ["u8", 32],
								},
								16,
							],
						},
					},
					{
						name: "s",
						type: {
							array: [
								{
									array: ["u8", 32],
								},
								16,
							],
						},
					},
					{
						name: "v",
						type: {
							array: ["u8", 16],
						},
					},
				],
			},
		},
	],
};
