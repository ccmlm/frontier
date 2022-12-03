//!
//!
//!

#![deny(warnings)]

use eth_pairings::public_interface::{perform_operation, ApiError, OperationType};
use fp_evm::{
	ExitError, ExitSucceed, Precompile, PrecompileFailure, PrecompileHandle, PrecompileOutput,
	PrecompileResult,
};
use precompile_utils::{EvmDataReader, EvmDataWriter};

const GAS_EXECUTION: u64 = 3437;

pub struct EthPairing;

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq, Eq, num_enum::TryFromPrimitive, num_enum::IntoPrimitive)]
pub enum Call {
	ExecuteOperation = "executeOperation()",
}

impl Precompile for EthPairing {
	fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult {
		let mut input = EvmDataReader::new(handle.input());
		let selector = input.read_selector::<Call>()?;

		match &selector {
			Call::ExecuteOperation => {
				let result = match Self::call_public_api_on_vector(input.get_slice()) {
					Ok(res) => res,
					Err(api_err) => {
						return Err(PrecompileFailure::Error {
							exit_status: ExitError::Other(api_err.to_string().into()),
						})
					}
				};

				handle.record_cost(GAS_EXECUTION)?;

				Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new()
						.write_raw_bytes(result.as_slice())
						.build(),
				})
			}
		}
	}
}

impl EthPairing {
	fn call_public_api_on_vector(data: &[u8]) -> Result<Vec<u8>, ApiError> {
		if data.is_empty() {
			return Err(ApiError::InputError("input length is zero".to_owned()));
		}
		let op = OperationType::from_u8(data[0]).ok_or(ApiError::MissingValue)?;
		perform_operation(op, &data[1..])
	}
}
