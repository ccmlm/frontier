//!
//!
//!

use eth_pairings::public_interface::{perform_operation, ApiError, OperationType};
use fp_evm::{
	Context, ExitError, ExitSucceed, Precompile, PrecompileFailure, PrecompileOutput,
	PrecompileResult,
};
use precompile_utils::{EvmDataReader, EvmDataWriter, EvmResult, Gasometer};

const GAS_EXECUTION: u64 = 3437;

pub struct EthPairing;

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq, Eq, num_enum::TryFromPrimitive, num_enum::IntoPrimitive)]
pub enum Call {
	ExecuteOperation = "executeOperation()",
}

impl Precompile for EthPairing {
	fn execute(
		input: &[u8],
		target_gas: Option<u64>,
		_context: &Context,
		_is_static: bool,
	) -> PrecompileResult {
		let mut input = EvmDataReader::new(input);
		let selector = input.read_selector::<Call>()?;

		match &selector {
			Call::ExecuteOperation => Self::execute_operation(input, target_gas),
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

	fn execute_operation(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(GAS_EXECUTION)?;

		let result = match Self::call_public_api_on_vector(input.get_slice()) {
			Ok(res) => res,
			Err(api_err) => {
				return Err(PrecompileFailure::Error {
					exit_status: ExitError::Other(api_err.to_string().into()),
				})
			}
		};

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new()
				.write_raw_bytes(result.as_slice())
				.build(),
			logs: vec![],
		})
	}
}
