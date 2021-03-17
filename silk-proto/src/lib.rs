mod protos;
pub use protos::*;

pub fn tx_validation_code_from(code: i32) -> TxValidationCode {
    match code {
        code if code == TxValidationCode::Valid as i32 => TxValidationCode::Valid,
        code if code == TxValidationCode::NilEnvelope as i32 => TxValidationCode::NilEnvelope,
        code if code == TxValidationCode::BadPayload as i32 => TxValidationCode::BadPayload,
        code if code == TxValidationCode::BadCommonHeader as i32 => {
            TxValidationCode::BadCommonHeader
        }
        code if code == TxValidationCode::BadCreatorSignature as i32 => {
            TxValidationCode::BadCreatorSignature
        }
        code if code == TxValidationCode::InvalidEndorserTransaction as i32 => {
            TxValidationCode::InvalidEndorserTransaction
        }
        code if code == TxValidationCode::InvalidConfigTransaction as i32 => {
            TxValidationCode::InvalidConfigTransaction
        }
        code if code == TxValidationCode::UnsupportedTxPayload as i32 => {
            TxValidationCode::UnsupportedTxPayload
        }
        code if code == TxValidationCode::BadProposalTxid as i32 => {
            TxValidationCode::BadProposalTxid
        }
        code if code == TxValidationCode::DuplicateTxid as i32 => TxValidationCode::DuplicateTxid,
        code if code == TxValidationCode::EndorsementPolicyFailure as i32 => {
            TxValidationCode::EndorsementPolicyFailure
        }
        code if code == TxValidationCode::MvccReadConflict as i32 => {
            TxValidationCode::MvccReadConflict
        }
        code if code == TxValidationCode::PhantomReadConflict as i32 => {
            TxValidationCode::PhantomReadConflict
        }
        code if code == TxValidationCode::UnknownTxType as i32 => TxValidationCode::UnknownTxType,
        code if code == TxValidationCode::TargetChainNotFound as i32 => {
            TxValidationCode::TargetChainNotFound
        }
        code if code == TxValidationCode::MarshalTxError as i32 => TxValidationCode::MarshalTxError,
        code if code == TxValidationCode::NilTxaction as i32 => TxValidationCode::NilTxaction,
        code if code == TxValidationCode::ExpiredChaincode as i32 => {
            TxValidationCode::ExpiredChaincode
        }
        code if code == TxValidationCode::ChaincodeVersionConflict as i32 => {
            TxValidationCode::ChaincodeVersionConflict
        }
        code if code == TxValidationCode::BadHeaderExtension as i32 => {
            TxValidationCode::BadHeaderExtension
        }
        code if code == TxValidationCode::BadChannelHeader as i32 => {
            TxValidationCode::BadChannelHeader
        }
        code if code == TxValidationCode::BadResponsePayload as i32 => {
            TxValidationCode::BadResponsePayload
        }
        code if code == TxValidationCode::BadRwset as i32 => TxValidationCode::BadRwset,
        code if code == TxValidationCode::IllegalWriteset as i32 => {
            TxValidationCode::IllegalWriteset
        }
        code if code == TxValidationCode::InvalidWriteset as i32 => {
            TxValidationCode::InvalidWriteset
        }
        code if code == TxValidationCode::InvalidChaincode as i32 => {
            TxValidationCode::InvalidChaincode
        }
        code if code == TxValidationCode::NotValidated as i32 => TxValidationCode::NotValidated,
        code if code == TxValidationCode::InvalidOtherReason as i32 => {
            TxValidationCode::InvalidOtherReason
        }
        _ => TxValidationCode::NilEnvelope,
    }
}
