use crate::core::traits::error::HasError;
use crate::core::traits::ibc::HasIbcTypes;

pub trait MessageHandler<Context>
where
    Context: HasIbcTypes + HasError,
{
    fn handle_message(context: &Context, message: &Context::Message) -> Result<(), Context::Error>;
}
