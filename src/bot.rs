use crate::ActionCaller;

use std::sync::Arc;
use walle_core::structs::Selft;

#[derive(Clone)]
pub struct Bot {
    pub selft: Selft,
    pub caller: Arc<dyn ActionCaller + Send + 'static>,
}
