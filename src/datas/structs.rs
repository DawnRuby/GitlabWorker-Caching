use crate::datas::enums;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RestoreData {
    pub(crate) restore_obj_name: String,
    pub(crate) cachetype: enums::CacheType,
    pub(crate) restore_to: String
}