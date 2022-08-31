#![allow(unused_qualifications)]

use crate::core_api::generated::models;

use crate::core_api::generated::header;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct CommittedTransaction {
    /// The resultant state version after the txn has been committed. A decimal 64-bit unsigned integer.
    #[serde(rename = "state_version")]
    pub state_version: String,

    #[serde(rename = "notarized_transaction")]
    pub notarized_transaction: models::NotarizedTransaction,

    #[serde(rename = "receipt")]
    pub receipt: models::TransactionReceipt,

}

impl CommittedTransaction {
    pub fn new(state_version: String, notarized_transaction: models::NotarizedTransaction, receipt: models::TransactionReceipt, ) -> CommittedTransaction {
        CommittedTransaction {
            state_version: state_version,
            notarized_transaction: notarized_transaction,
            receipt: receipt,
        }
    }
}

/// Converts the CommittedTransaction value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for CommittedTransaction {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("state_version".to_string());
        params.push(self.state_version.to_string());

        // Skipping notarized_transaction in query parameter serialization

        // Skipping receipt in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a CommittedTransaction value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for CommittedTransaction {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub state_version: Vec<String>,
            pub notarized_transaction: Vec<models::NotarizedTransaction>,
            pub receipt: Vec<models::TransactionReceipt>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing CommittedTransaction".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "state_version" => intermediate_rep.state_version.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "notarized_transaction" => intermediate_rep.notarized_transaction.push(<models::NotarizedTransaction as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "receipt" => intermediate_rep.receipt.push(<models::TransactionReceipt as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing CommittedTransaction".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(CommittedTransaction {
            state_version: intermediate_rep.state_version.into_iter().next().ok_or("state_version missing in CommittedTransaction".to_string())?,
            notarized_transaction: intermediate_rep.notarized_transaction.into_iter().next().ok_or("notarized_transaction missing in CommittedTransaction".to_string())?,
            receipt: intermediate_rep.receipt.into_iter().next().ok_or("receipt missing in CommittedTransaction".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<CommittedTransaction> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<CommittedTransaction>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<CommittedTransaction>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for CommittedTransaction - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<CommittedTransaction> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <CommittedTransaction as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into CommittedTransaction - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


/// A request to retrieve a sublist of committed transactions from the ledger.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct CommittedTransactionsRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: models::NetworkIdentifier,

    /// A decimal 64-bit unsigned integer.
    #[serde(rename = "start_state_version")]
    pub start_state_version: String,

    /// The maximum number of transactions that will be returned.
    #[serde(rename = "limit")]
    pub limit: isize,

}

impl CommittedTransactionsRequest {
    pub fn new(network_identifier: models::NetworkIdentifier, start_state_version: String, limit: isize, ) -> CommittedTransactionsRequest {
        CommittedTransactionsRequest {
            network_identifier: network_identifier,
            start_state_version: start_state_version,
            limit: limit,
        }
    }
}

/// Converts the CommittedTransactionsRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for CommittedTransactionsRequest {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping network_identifier in query parameter serialization


        params.push("start_state_version".to_string());
        params.push(self.start_state_version.to_string());


        params.push("limit".to_string());
        params.push(self.limit.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a CommittedTransactionsRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for CommittedTransactionsRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub network_identifier: Vec<models::NetworkIdentifier>,
            pub start_state_version: Vec<String>,
            pub limit: Vec<isize>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing CommittedTransactionsRequest".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "network_identifier" => intermediate_rep.network_identifier.push(<models::NetworkIdentifier as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "start_state_version" => intermediate_rep.start_state_version.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "limit" => intermediate_rep.limit.push(<isize as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing CommittedTransactionsRequest".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(CommittedTransactionsRequest {
            network_identifier: intermediate_rep.network_identifier.into_iter().next().ok_or("network_identifier missing in CommittedTransactionsRequest".to_string())?,
            start_state_version: intermediate_rep.start_state_version.into_iter().next().ok_or("start_state_version missing in CommittedTransactionsRequest".to_string())?,
            limit: intermediate_rep.limit.into_iter().next().ok_or("limit missing in CommittedTransactionsRequest".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<CommittedTransactionsRequest> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<CommittedTransactionsRequest>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<CommittedTransactionsRequest>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for CommittedTransactionsRequest - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<CommittedTransactionsRequest> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <CommittedTransactionsRequest as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into CommittedTransactionsRequest - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct CommittedTransactionsResponse {
    /// The first state version returned. A decimal 64-bit unsigned integer.
    #[serde(rename = "start_state_version")]
    pub start_state_version: String,

    /// The maximum state version returned. A decimal 64-bit unsigned integer.
    #[serde(rename = "max_state_version")]
    pub max_state_version: String,

    /// A committed transactions list starting from the `start_state_version_inclusive`.
    #[serde(rename = "transactions")]
    pub transactions: Vec<models::CommittedTransaction>,

}

impl CommittedTransactionsResponse {
    pub fn new(start_state_version: String, max_state_version: String, transactions: Vec<models::CommittedTransaction>, ) -> CommittedTransactionsResponse {
        CommittedTransactionsResponse {
            start_state_version: start_state_version,
            max_state_version: max_state_version,
            transactions: transactions,
        }
    }
}

/// Converts the CommittedTransactionsResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for CommittedTransactionsResponse {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("start_state_version".to_string());
        params.push(self.start_state_version.to_string());


        params.push("max_state_version".to_string());
        params.push(self.max_state_version.to_string());

        // Skipping transactions in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a CommittedTransactionsResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for CommittedTransactionsResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub start_state_version: Vec<String>,
            pub max_state_version: Vec<String>,
            pub transactions: Vec<Vec<models::CommittedTransaction>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing CommittedTransactionsResponse".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "start_state_version" => intermediate_rep.start_state_version.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "max_state_version" => intermediate_rep.max_state_version.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "transactions" => return std::result::Result::Err("Parsing a container in this style is not supported in CommittedTransactionsResponse".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing CommittedTransactionsResponse".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(CommittedTransactionsResponse {
            start_state_version: intermediate_rep.start_state_version.into_iter().next().ok_or("start_state_version missing in CommittedTransactionsResponse".to_string())?,
            max_state_version: intermediate_rep.max_state_version.into_iter().next().ok_or("max_state_version missing in CommittedTransactionsResponse".to_string())?,
            transactions: intermediate_rep.transactions.into_iter().next().ok_or("transactions missing in CommittedTransactionsResponse".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<CommittedTransactionsResponse> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<CommittedTransactionsResponse>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<CommittedTransactionsResponse>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for CommittedTransactionsResponse - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<CommittedTransactionsResponse> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <CommittedTransactionsResponse as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into CommittedTransactionsResponse - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


/// Bech32 component address.
#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ComponentAddress(String);

impl std::convert::From<String> for ComponentAddress {
    fn from(x: String) -> Self {
        ComponentAddress(x)
    }
}

impl std::string::ToString for ComponentAddress {
    fn to_string(&self) -> String {
       self.0.to_string()
    }
}

impl std::str::FromStr for ComponentAddress {
    type Err = std::string::ParseError;
    fn from_str(x: &str) -> std::result::Result<Self, Self::Err> {
        std::result::Result::Ok(ComponentAddress(x.to_string()))
    }
}

impl std::convert::From<ComponentAddress> for String {
    fn from(x: ComponentAddress) -> Self {
        x.0
    }
}

impl std::ops::Deref for ComponentAddress {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl std::ops::DerefMut for ComponentAddress {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ComponentInfoSubstate {
    /// Package address, bech32-encoded.
    #[serde(rename = "package_address")]
    pub package_address: String,

    #[serde(rename = "blueprint_name")]
    pub blueprint_name: String,

}

impl ComponentInfoSubstate {
    pub fn new(package_address: String, blueprint_name: String, ) -> ComponentInfoSubstate {
        ComponentInfoSubstate {
            package_address: package_address,
            blueprint_name: blueprint_name,
        }
    }
}

/// Converts the ComponentInfoSubstate value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ComponentInfoSubstate {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("package_address".to_string());
        params.push(self.package_address.to_string());


        params.push("blueprint_name".to_string());
        params.push(self.blueprint_name.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ComponentInfoSubstate value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ComponentInfoSubstate {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub package_address: Vec<String>,
            pub blueprint_name: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing ComponentInfoSubstate".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "package_address" => intermediate_rep.package_address.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "blueprint_name" => intermediate_rep.blueprint_name.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing ComponentInfoSubstate".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ComponentInfoSubstate {
            package_address: intermediate_rep.package_address.into_iter().next().ok_or("package_address missing in ComponentInfoSubstate".to_string())?,
            blueprint_name: intermediate_rep.blueprint_name.into_iter().next().ok_or("blueprint_name missing in ComponentInfoSubstate".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ComponentInfoSubstate> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<ComponentInfoSubstate>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<ComponentInfoSubstate>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for ComponentInfoSubstate - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<ComponentInfoSubstate> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <ComponentInfoSubstate as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into ComponentInfoSubstate - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ComponentStateSubstate {
    /// hex-encoded state data
    #[serde(rename = "state")]
    pub state: String,

}

impl ComponentStateSubstate {
    pub fn new(state: String, ) -> ComponentStateSubstate {
        ComponentStateSubstate {
            state: state,
        }
    }
}

/// Converts the ComponentStateSubstate value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ComponentStateSubstate {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("state".to_string());
        params.push(self.state.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ComponentStateSubstate value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ComponentStateSubstate {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub state: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing ComponentStateSubstate".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "state" => intermediate_rep.state.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing ComponentStateSubstate".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ComponentStateSubstate {
            state: intermediate_rep.state.into_iter().next().ok_or("state missing in ComponentStateSubstate".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ComponentStateSubstate> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<ComponentStateSubstate>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<ComponentStateSubstate>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for ComponentStateSubstate - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<ComponentStateSubstate> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <ComponentStateSubstate as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into ComponentStateSubstate - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct DownSubstate {
    /// SBOR-encoded and then hex-encoded substate ID.
    #[serde(rename = "substate_id")]
    pub substate_id: String,

    /// Substate hash.
    #[serde(rename = "substate_hash")]
    pub substate_hash: String,

    /// A decimal 32-bit unsigned integer
    #[serde(rename = "version")]
    pub version: String,

}

impl DownSubstate {
    pub fn new(substate_id: String, substate_hash: String, version: String, ) -> DownSubstate {
        DownSubstate {
            substate_id: substate_id,
            substate_hash: substate_hash,
            version: version,
        }
    }
}

/// Converts the DownSubstate value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for DownSubstate {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("substate_id".to_string());
        params.push(self.substate_id.to_string());


        params.push("substate_hash".to_string());
        params.push(self.substate_hash.to_string());


        params.push("version".to_string());
        params.push(self.version.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a DownSubstate value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for DownSubstate {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub substate_id: Vec<String>,
            pub substate_hash: Vec<String>,
            pub version: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing DownSubstate".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "substate_id" => intermediate_rep.substate_id.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "substate_hash" => intermediate_rep.substate_hash.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "version" => intermediate_rep.version.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing DownSubstate".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(DownSubstate {
            substate_id: intermediate_rep.substate_id.into_iter().next().ok_or("substate_id missing in DownSubstate".to_string())?,
            substate_hash: intermediate_rep.substate_hash.into_iter().next().ok_or("substate_hash missing in DownSubstate".to_string())?,
            version: intermediate_rep.version.into_iter().next().ok_or("version missing in DownSubstate".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<DownSubstate> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<DownSubstate>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<DownSubstate>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for DownSubstate - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<DownSubstate> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <DownSubstate as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into DownSubstate - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


/// A not yet implemented substate model
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct EmptySubstate {
    #[serde(rename = "dummy")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub dummy: Option<String>,

}

impl EmptySubstate {
    pub fn new() -> EmptySubstate {
        EmptySubstate {
            dummy: None,
        }
    }
}

/// Converts the EmptySubstate value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for EmptySubstate {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref dummy) = self.dummy {
            params.push("dummy".to_string());
            params.push(dummy.to_string());
        }

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a EmptySubstate value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for EmptySubstate {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub dummy: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing EmptySubstate".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "dummy" => intermediate_rep.dummy.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing EmptySubstate".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(EmptySubstate {
            dummy: intermediate_rep.dummy.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<EmptySubstate> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<EmptySubstate>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<EmptySubstate>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for EmptySubstate - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<EmptySubstate> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <EmptySubstate as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into EmptySubstate - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ErrorResponse {
    /// A numeric code corresponding to the given error type.
    #[serde(rename = "code")]
    pub code: isize,

    /// A human-readable error message.
    #[serde(rename = "message")]
    pub message: String,

    /// A GUID to be used when reporting errors, to allow correlation with the Core API's error logs, in the case where the Core API details are hidden.
    #[serde(rename = "trace_id")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub trace_id: Option<String>,

}

impl ErrorResponse {
    pub fn new(code: isize, message: String, ) -> ErrorResponse {
        ErrorResponse {
            code: code,
            message: message,
            trace_id: None,
        }
    }
}

/// Converts the ErrorResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ErrorResponse {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("code".to_string());
        params.push(self.code.to_string());


        params.push("message".to_string());
        params.push(self.message.to_string());


        if let Some(ref trace_id) = self.trace_id {
            params.push("trace_id".to_string());
            params.push(trace_id.to_string());
        }

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ErrorResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ErrorResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub code: Vec<isize>,
            pub message: Vec<String>,
            pub trace_id: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing ErrorResponse".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "code" => intermediate_rep.code.push(<isize as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "message" => intermediate_rep.message.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "trace_id" => intermediate_rep.trace_id.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing ErrorResponse".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ErrorResponse {
            code: intermediate_rep.code.into_iter().next().ok_or("code missing in ErrorResponse".to_string())?,
            message: intermediate_rep.message.into_iter().next().ok_or("message missing in ErrorResponse".to_string())?,
            trace_id: intermediate_rep.trace_id.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ErrorResponse> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<ErrorResponse>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<ErrorResponse>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for ErrorResponse - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<ErrorResponse> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <ErrorResponse as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into ErrorResponse - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


/// Fees paid
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct FeeSummary {
    /// Specifies whether the transaction execution loan has been fully repaid.
    #[serde(rename = "loan_fully_repaid")]
    pub loan_fully_repaid: bool,

    /// Maximum amount of cost units available for the transaction execution. A decimal 32-bit unsigned integer.
    #[serde(rename = "cost_unit_limit")]
    pub cost_unit_limit: String,

    /// The amount of cost units consumed by the transaction execution. A decimal 32-bit unsigned integer.
    #[serde(rename = "cost_unit_consumed")]
    pub cost_unit_consumed: String,

    /// The XRD price of a single cost unit. A fixed-scale 256-bit signed decimal number.
    #[serde(rename = "cost_unit_price")]
    pub cost_unit_price: String,

    /// The validator tip. A decimal 32-bit unsigned integer, representing the percentage amount (a value of \"1\" corresponds to 1%).
    #[serde(rename = "tip_percentage")]
    pub tip_percentage: String,

    /// The total amount of XRD burned. A fixed-scale 256-bit signed decimal number.
    #[serde(rename = "xrd_burned")]
    pub xrd_burned: String,

    /// The total amount of XRD tipped to validators. A fixed-scale 256-bit signed decimal number.
    #[serde(rename = "xrd_tipped")]
    pub xrd_tipped: String,

}

impl FeeSummary {
    pub fn new(loan_fully_repaid: bool, cost_unit_limit: String, cost_unit_consumed: String, cost_unit_price: String, tip_percentage: String, xrd_burned: String, xrd_tipped: String, ) -> FeeSummary {
        FeeSummary {
            loan_fully_repaid: loan_fully_repaid,
            cost_unit_limit: cost_unit_limit,
            cost_unit_consumed: cost_unit_consumed,
            cost_unit_price: cost_unit_price,
            tip_percentage: tip_percentage,
            xrd_burned: xrd_burned,
            xrd_tipped: xrd_tipped,
        }
    }
}

/// Converts the FeeSummary value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for FeeSummary {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("loan_fully_repaid".to_string());
        params.push(self.loan_fully_repaid.to_string());


        params.push("cost_unit_limit".to_string());
        params.push(self.cost_unit_limit.to_string());


        params.push("cost_unit_consumed".to_string());
        params.push(self.cost_unit_consumed.to_string());


        params.push("cost_unit_price".to_string());
        params.push(self.cost_unit_price.to_string());


        params.push("tip_percentage".to_string());
        params.push(self.tip_percentage.to_string());


        params.push("xrd_burned".to_string());
        params.push(self.xrd_burned.to_string());


        params.push("xrd_tipped".to_string());
        params.push(self.xrd_tipped.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a FeeSummary value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for FeeSummary {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub loan_fully_repaid: Vec<bool>,
            pub cost_unit_limit: Vec<String>,
            pub cost_unit_consumed: Vec<String>,
            pub cost_unit_price: Vec<String>,
            pub tip_percentage: Vec<String>,
            pub xrd_burned: Vec<String>,
            pub xrd_tipped: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing FeeSummary".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "loan_fully_repaid" => intermediate_rep.loan_fully_repaid.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "cost_unit_limit" => intermediate_rep.cost_unit_limit.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "cost_unit_consumed" => intermediate_rep.cost_unit_consumed.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "cost_unit_price" => intermediate_rep.cost_unit_price.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "tip_percentage" => intermediate_rep.tip_percentage.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "xrd_burned" => intermediate_rep.xrd_burned.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "xrd_tipped" => intermediate_rep.xrd_tipped.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing FeeSummary".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(FeeSummary {
            loan_fully_repaid: intermediate_rep.loan_fully_repaid.into_iter().next().ok_or("loan_fully_repaid missing in FeeSummary".to_string())?,
            cost_unit_limit: intermediate_rep.cost_unit_limit.into_iter().next().ok_or("cost_unit_limit missing in FeeSummary".to_string())?,
            cost_unit_consumed: intermediate_rep.cost_unit_consumed.into_iter().next().ok_or("cost_unit_consumed missing in FeeSummary".to_string())?,
            cost_unit_price: intermediate_rep.cost_unit_price.into_iter().next().ok_or("cost_unit_price missing in FeeSummary".to_string())?,
            tip_percentage: intermediate_rep.tip_percentage.into_iter().next().ok_or("tip_percentage missing in FeeSummary".to_string())?,
            xrd_burned: intermediate_rep.xrd_burned.into_iter().next().ok_or("xrd_burned missing in FeeSummary".to_string())?,
            xrd_tipped: intermediate_rep.xrd_tipped.into_iter().next().ok_or("xrd_tipped missing in FeeSummary".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<FeeSummary> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<FeeSummary>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<FeeSummary>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for FeeSummary - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<FeeSummary> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <FeeSummary as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into FeeSummary - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct IntentSignature {
    /// Signer public key, hex-encoded.
    #[serde(rename = "public_key")]
    pub public_key: String,

    /// The signature, hex-encoded.
    #[serde(rename = "signature")]
    pub signature: String,

}

impl IntentSignature {
    pub fn new(public_key: String, signature: String, ) -> IntentSignature {
        IntentSignature {
            public_key: public_key,
            signature: signature,
        }
    }
}

/// Converts the IntentSignature value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for IntentSignature {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("public_key".to_string());
        params.push(self.public_key.to_string());


        params.push("signature".to_string());
        params.push(self.signature.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a IntentSignature value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for IntentSignature {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub public_key: Vec<String>,
            pub signature: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing IntentSignature".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "public_key" => intermediate_rep.public_key.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "signature" => intermediate_rep.signature.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing IntentSignature".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(IntentSignature {
            public_key: intermediate_rep.public_key.into_iter().next().ok_or("public_key missing in IntentSignature".to_string())?,
            signature: intermediate_rep.signature.into_iter().next().ok_or("signature missing in IntentSignature".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<IntentSignature> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<IntentSignature>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<IntentSignature>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for IntentSignature - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<IntentSignature> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <IntentSignature as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into IntentSignature - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NetworkConfigurationResponse {
    #[serde(rename = "version")]
    pub version: models::NetworkConfigurationResponseVersion,

    #[serde(rename = "network_identifier")]
    pub network_identifier: models::NetworkIdentifier,

    /// The network suffix used for bech32 hrps used for addressing.
    #[serde(rename = "network_hrp_suffix")]
    pub network_hrp_suffix: String,

}

impl NetworkConfigurationResponse {
    pub fn new(version: models::NetworkConfigurationResponseVersion, network_identifier: models::NetworkIdentifier, network_hrp_suffix: String, ) -> NetworkConfigurationResponse {
        NetworkConfigurationResponse {
            version: version,
            network_identifier: network_identifier,
            network_hrp_suffix: network_hrp_suffix,
        }
    }
}

/// Converts the NetworkConfigurationResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for NetworkConfigurationResponse {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping version in query parameter serialization

        // Skipping network_identifier in query parameter serialization


        params.push("network_hrp_suffix".to_string());
        params.push(self.network_hrp_suffix.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a NetworkConfigurationResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for NetworkConfigurationResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub version: Vec<models::NetworkConfigurationResponseVersion>,
            pub network_identifier: Vec<models::NetworkIdentifier>,
            pub network_hrp_suffix: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing NetworkConfigurationResponse".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "version" => intermediate_rep.version.push(<models::NetworkConfigurationResponseVersion as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "network_identifier" => intermediate_rep.network_identifier.push(<models::NetworkIdentifier as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "network_hrp_suffix" => intermediate_rep.network_hrp_suffix.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing NetworkConfigurationResponse".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(NetworkConfigurationResponse {
            version: intermediate_rep.version.into_iter().next().ok_or("version missing in NetworkConfigurationResponse".to_string())?,
            network_identifier: intermediate_rep.network_identifier.into_iter().next().ok_or("network_identifier missing in NetworkConfigurationResponse".to_string())?,
            network_hrp_suffix: intermediate_rep.network_hrp_suffix.into_iter().next().ok_or("network_hrp_suffix missing in NetworkConfigurationResponse".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<NetworkConfigurationResponse> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<NetworkConfigurationResponse>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<NetworkConfigurationResponse>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for NetworkConfigurationResponse - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<NetworkConfigurationResponse> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <NetworkConfigurationResponse as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into NetworkConfigurationResponse - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


/// Different versions regarding the node, network and api.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NetworkConfigurationResponseVersion {
    #[serde(rename = "core_version")]
    pub core_version: String,

    #[serde(rename = "api_version")]
    pub api_version: String,

}

impl NetworkConfigurationResponseVersion {
    pub fn new(core_version: String, api_version: String, ) -> NetworkConfigurationResponseVersion {
        NetworkConfigurationResponseVersion {
            core_version: core_version,
            api_version: api_version,
        }
    }
}

/// Converts the NetworkConfigurationResponseVersion value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for NetworkConfigurationResponseVersion {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("core_version".to_string());
        params.push(self.core_version.to_string());


        params.push("api_version".to_string());
        params.push(self.api_version.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a NetworkConfigurationResponseVersion value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for NetworkConfigurationResponseVersion {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub core_version: Vec<String>,
            pub api_version: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing NetworkConfigurationResponseVersion".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "core_version" => intermediate_rep.core_version.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "api_version" => intermediate_rep.api_version.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing NetworkConfigurationResponseVersion".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(NetworkConfigurationResponseVersion {
            core_version: intermediate_rep.core_version.into_iter().next().ok_or("core_version missing in NetworkConfigurationResponseVersion".to_string())?,
            api_version: intermediate_rep.api_version.into_iter().next().ok_or("api_version missing in NetworkConfigurationResponseVersion".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<NetworkConfigurationResponseVersion> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<NetworkConfigurationResponseVersion>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<NetworkConfigurationResponseVersion>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for NetworkConfigurationResponseVersion - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<NetworkConfigurationResponseVersion> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <NetworkConfigurationResponseVersion as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into NetworkConfigurationResponseVersion - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


/// The name of the network.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NetworkIdentifier {
    #[serde(rename = "network")]
    pub network: String,

}

impl NetworkIdentifier {
    pub fn new(network: String, ) -> NetworkIdentifier {
        NetworkIdentifier {
            network: network,
        }
    }
}

/// Converts the NetworkIdentifier value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for NetworkIdentifier {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("network".to_string());
        params.push(self.network.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a NetworkIdentifier value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for NetworkIdentifier {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub network: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing NetworkIdentifier".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "network" => intermediate_rep.network.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing NetworkIdentifier".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(NetworkIdentifier {
            network: intermediate_rep.network.into_iter().next().ok_or("network missing in NetworkIdentifier".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<NetworkIdentifier> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<NetworkIdentifier>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<NetworkIdentifier>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for NetworkIdentifier - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<NetworkIdentifier> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <NetworkIdentifier as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into NetworkIdentifier - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NotarizedTransaction {
    /// The transaction hash, hex-encoded.
    #[serde(rename = "hash")]
    pub hash: String,

    #[serde(rename = "signed_intent")]
    pub signed_intent: models::SignedTransactionIntent,

    /// The notary signature, hex-encoded.
    #[serde(rename = "notary_signature")]
    pub notary_signature: String,

}

impl NotarizedTransaction {
    pub fn new(hash: String, signed_intent: models::SignedTransactionIntent, notary_signature: String, ) -> NotarizedTransaction {
        NotarizedTransaction {
            hash: hash,
            signed_intent: signed_intent,
            notary_signature: notary_signature,
        }
    }
}

/// Converts the NotarizedTransaction value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for NotarizedTransaction {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("hash".to_string());
        params.push(self.hash.to_string());

        // Skipping signed_intent in query parameter serialization


        params.push("notary_signature".to_string());
        params.push(self.notary_signature.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a NotarizedTransaction value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for NotarizedTransaction {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub hash: Vec<String>,
            pub signed_intent: Vec<models::SignedTransactionIntent>,
            pub notary_signature: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing NotarizedTransaction".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "hash" => intermediate_rep.hash.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "signed_intent" => intermediate_rep.signed_intent.push(<models::SignedTransactionIntent as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "notary_signature" => intermediate_rep.notary_signature.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing NotarizedTransaction".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(NotarizedTransaction {
            hash: intermediate_rep.hash.into_iter().next().ok_or("hash missing in NotarizedTransaction".to_string())?,
            signed_intent: intermediate_rep.signed_intent.into_iter().next().ok_or("signed_intent missing in NotarizedTransaction".to_string())?,
            notary_signature: intermediate_rep.notary_signature.into_iter().next().ok_or("notary_signature missing in NotarizedTransaction".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<NotarizedTransaction> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<NotarizedTransaction>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<NotarizedTransaction>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for NotarizedTransaction - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<NotarizedTransaction> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <NotarizedTransaction as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into NotarizedTransaction - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


/// Bech32 package address.
#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct PackageAddress(String);

impl std::convert::From<String> for PackageAddress {
    fn from(x: String) -> Self {
        PackageAddress(x)
    }
}

impl std::string::ToString for PackageAddress {
    fn to_string(&self) -> String {
       self.0.to_string()
    }
}

impl std::str::FromStr for PackageAddress {
    type Err = std::string::ParseError;
    fn from_str(x: &str) -> std::result::Result<Self, Self::Err> {
        std::result::Result::Ok(PackageAddress(x.to_string()))
    }
}

impl std::convert::From<PackageAddress> for String {
    fn from(x: PackageAddress) -> Self {
        x.0
    }
}

impl std::ops::Deref for PackageAddress {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl std::ops::DerefMut for PackageAddress {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct PackageSubstate {
    /// Package code, hex-encoded.
    #[serde(rename = "code")]
    pub code: String,

}

impl PackageSubstate {
    pub fn new(code: String, ) -> PackageSubstate {
        PackageSubstate {
            code: code,
        }
    }
}

/// Converts the PackageSubstate value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for PackageSubstate {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("code".to_string());
        params.push(self.code.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a PackageSubstate value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for PackageSubstate {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub code: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing PackageSubstate".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "code" => intermediate_rep.code.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing PackageSubstate".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(PackageSubstate {
            code: intermediate_rep.code.into_iter().next().ok_or("code missing in PackageSubstate".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<PackageSubstate> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<PackageSubstate>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<PackageSubstate>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for PackageSubstate - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<PackageSubstate> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <PackageSubstate as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into PackageSubstate - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


/// Bech32 resource address.
#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ResourceAddress(String);

impl std::convert::From<String> for ResourceAddress {
    fn from(x: String) -> Self {
        ResourceAddress(x)
    }
}

impl std::string::ToString for ResourceAddress {
    fn to_string(&self) -> String {
       self.0.to_string()
    }
}

impl std::str::FromStr for ResourceAddress {
    type Err = std::string::ParseError;
    fn from_str(x: &str) -> std::result::Result<Self, Self::Err> {
        std::result::Result::Ok(ResourceAddress(x.to_string()))
    }
}

impl std::convert::From<ResourceAddress> for String {
    fn from(x: ResourceAddress) -> Self {
        x.0
    }
}

impl std::ops::Deref for ResourceAddress {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl std::ops::DerefMut for ResourceAddress {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ResourceChange {
    /// Bech32 resource address.
    #[serde(rename = "resource_address")]
    pub resource_address: String,

    /// Bech32 component address.
    #[serde(rename = "component_address")]
    pub component_address: String,

    /// Vault ID, SBOR-encoded and then hex-encoded.
    #[serde(rename = "vault_id")]
    pub vault_id: String,

    /// The XRD amount put or taken from the vault. A fixed-scale 256-bit signed decimal number.
    #[serde(rename = "amount")]
    pub amount: String,

}

impl ResourceChange {
    pub fn new(resource_address: String, component_address: String, vault_id: String, amount: String, ) -> ResourceChange {
        ResourceChange {
            resource_address: resource_address,
            component_address: component_address,
            vault_id: vault_id,
            amount: amount,
        }
    }
}

/// Converts the ResourceChange value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ResourceChange {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("resource_address".to_string());
        params.push(self.resource_address.to_string());


        params.push("component_address".to_string());
        params.push(self.component_address.to_string());


        params.push("vault_id".to_string());
        params.push(self.vault_id.to_string());


        params.push("amount".to_string());
        params.push(self.amount.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ResourceChange value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ResourceChange {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub resource_address: Vec<String>,
            pub component_address: Vec<String>,
            pub vault_id: Vec<String>,
            pub amount: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing ResourceChange".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "resource_address" => intermediate_rep.resource_address.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "component_address" => intermediate_rep.component_address.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "vault_id" => intermediate_rep.vault_id.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "amount" => intermediate_rep.amount.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing ResourceChange".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ResourceChange {
            resource_address: intermediate_rep.resource_address.into_iter().next().ok_or("resource_address missing in ResourceChange".to_string())?,
            component_address: intermediate_rep.component_address.into_iter().next().ok_or("component_address missing in ResourceChange".to_string())?,
            vault_id: intermediate_rep.vault_id.into_iter().next().ok_or("vault_id missing in ResourceChange".to_string())?,
            amount: intermediate_rep.amount.into_iter().next().ok_or("amount missing in ResourceChange".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ResourceChange> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<ResourceChange>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<ResourceChange>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for ResourceChange - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<ResourceChange> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <ResourceChange as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into ResourceChange - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ResourceSubstate {
    // Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "resource_type")]
    pub resource_type: String,

    #[serde(rename = "fungible_divisibility")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub fungible_divisibility: Option<isize>,

    #[serde(rename = "metadata")]
    pub metadata: Vec<models::ResourceSubstateMetadataInner>,

    #[serde(rename = "total_supply")]
    pub total_supply: String,

}

impl ResourceSubstate {
    pub fn new(resource_type: String, metadata: Vec<models::ResourceSubstateMetadataInner>, total_supply: String, ) -> ResourceSubstate {
        ResourceSubstate {
            resource_type: resource_type,
            fungible_divisibility: None,
            metadata: metadata,
            total_supply: total_supply,
        }
    }
}

/// Converts the ResourceSubstate value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ResourceSubstate {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("resource_type".to_string());
        params.push(self.resource_type.to_string());


        if let Some(ref fungible_divisibility) = self.fungible_divisibility {
            params.push("fungible_divisibility".to_string());
            params.push(fungible_divisibility.to_string());
        }

        // Skipping metadata in query parameter serialization


        params.push("total_supply".to_string());
        params.push(self.total_supply.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ResourceSubstate value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ResourceSubstate {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub resource_type: Vec<String>,
            pub fungible_divisibility: Vec<isize>,
            pub metadata: Vec<Vec<models::ResourceSubstateMetadataInner>>,
            pub total_supply: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing ResourceSubstate".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "resource_type" => intermediate_rep.resource_type.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "fungible_divisibility" => intermediate_rep.fungible_divisibility.push(<isize as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "metadata" => return std::result::Result::Err("Parsing a container in this style is not supported in ResourceSubstate".to_string()),
                    "total_supply" => intermediate_rep.total_supply.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing ResourceSubstate".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ResourceSubstate {
            resource_type: intermediate_rep.resource_type.into_iter().next().ok_or("resource_type missing in ResourceSubstate".to_string())?,
            fungible_divisibility: intermediate_rep.fungible_divisibility.into_iter().next(),
            metadata: intermediate_rep.metadata.into_iter().next().ok_or("metadata missing in ResourceSubstate".to_string())?,
            total_supply: intermediate_rep.total_supply.into_iter().next().ok_or("total_supply missing in ResourceSubstate".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ResourceSubstate> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<ResourceSubstate>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<ResourceSubstate>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for ResourceSubstate - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<ResourceSubstate> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <ResourceSubstate as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into ResourceSubstate - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ResourceSubstateMetadataInner {
    #[serde(rename = "key")]
    pub key: String,

    #[serde(rename = "value")]
    pub value: String,

}

impl ResourceSubstateMetadataInner {
    pub fn new(key: String, value: String, ) -> ResourceSubstateMetadataInner {
        ResourceSubstateMetadataInner {
            key: key,
            value: value,
        }
    }
}

/// Converts the ResourceSubstateMetadataInner value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ResourceSubstateMetadataInner {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("key".to_string());
        params.push(self.key.to_string());


        params.push("value".to_string());
        params.push(self.value.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ResourceSubstateMetadataInner value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ResourceSubstateMetadataInner {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub key: Vec<String>,
            pub value: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing ResourceSubstateMetadataInner".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "key" => intermediate_rep.key.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "value" => intermediate_rep.value.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing ResourceSubstateMetadataInner".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ResourceSubstateMetadataInner {
            key: intermediate_rep.key.into_iter().next().ok_or("key missing in ResourceSubstateMetadataInner".to_string())?,
            value: intermediate_rep.value.into_iter().next().ok_or("value missing in ResourceSubstateMetadataInner".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ResourceSubstateMetadataInner> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<ResourceSubstateMetadataInner>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<ResourceSubstateMetadataInner>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for ResourceSubstateMetadataInner - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<ResourceSubstateMetadataInner> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <ResourceSubstateMetadataInner as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into ResourceSubstateMetadataInner - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct SignedTransactionIntent {
    /// Signed transaction intent hash, hex-encoded.
    #[serde(rename = "hash")]
    pub hash: String,

    #[serde(rename = "intent")]
    pub intent: models::TransactionIntent,

    #[serde(rename = "intent_signatures")]
    pub intent_signatures: Vec<models::IntentSignature>,

}

impl SignedTransactionIntent {
    pub fn new(hash: String, intent: models::TransactionIntent, intent_signatures: Vec<models::IntentSignature>, ) -> SignedTransactionIntent {
        SignedTransactionIntent {
            hash: hash,
            intent: intent,
            intent_signatures: intent_signatures,
        }
    }
}

/// Converts the SignedTransactionIntent value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for SignedTransactionIntent {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("hash".to_string());
        params.push(self.hash.to_string());

        // Skipping intent in query parameter serialization

        // Skipping intent_signatures in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a SignedTransactionIntent value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for SignedTransactionIntent {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub hash: Vec<String>,
            pub intent: Vec<models::TransactionIntent>,
            pub intent_signatures: Vec<Vec<models::IntentSignature>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing SignedTransactionIntent".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "hash" => intermediate_rep.hash.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "intent" => intermediate_rep.intent.push(<models::TransactionIntent as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "intent_signatures" => return std::result::Result::Err("Parsing a container in this style is not supported in SignedTransactionIntent".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing SignedTransactionIntent".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(SignedTransactionIntent {
            hash: intermediate_rep.hash.into_iter().next().ok_or("hash missing in SignedTransactionIntent".to_string())?,
            intent: intermediate_rep.intent.into_iter().next().ok_or("intent missing in SignedTransactionIntent".to_string())?,
            intent_signatures: intermediate_rep.intent_signatures.into_iter().next().ok_or("intent_signatures missing in SignedTransactionIntent".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<SignedTransactionIntent> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<SignedTransactionIntent>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<SignedTransactionIntent>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for SignedTransactionIntent - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<SignedTransactionIntent> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <SignedTransactionIntent as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into SignedTransactionIntent - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


/// Transaction state updates
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct StateUpdates {
    #[serde(rename = "down_virtual_substates")]
    pub down_virtual_substates: Vec<models::VirtualSubstateId>,

    #[serde(rename = "up_substates")]
    pub up_substates: Vec<models::UpSubstate>,

    #[serde(rename = "down_substates")]
    pub down_substates: Vec<models::DownSubstate>,

    #[serde(rename = "new_roots")]
    pub new_roots: Vec<models::SubstateId>,

}

impl StateUpdates {
    pub fn new(down_virtual_substates: Vec<models::VirtualSubstateId>, up_substates: Vec<models::UpSubstate>, down_substates: Vec<models::DownSubstate>, new_roots: Vec<models::SubstateId>, ) -> StateUpdates {
        StateUpdates {
            down_virtual_substates: down_virtual_substates,
            up_substates: up_substates,
            down_substates: down_substates,
            new_roots: new_roots,
        }
    }
}

/// Converts the StateUpdates value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for StateUpdates {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("down_virtual_substates".to_string());
        params.push(self.down_virtual_substates.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",").to_string());

        // Skipping up_substates in query parameter serialization

        // Skipping down_substates in query parameter serialization


        params.push("new_roots".to_string());
        params.push(self.new_roots.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",").to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a StateUpdates value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for StateUpdates {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub down_virtual_substates: Vec<Vec<models::VirtualSubstateId>>,
            pub up_substates: Vec<Vec<models::UpSubstate>>,
            pub down_substates: Vec<Vec<models::DownSubstate>>,
            pub new_roots: Vec<Vec<models::SubstateId>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing StateUpdates".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "down_virtual_substates" => return std::result::Result::Err("Parsing a container in this style is not supported in StateUpdates".to_string()),
                    "up_substates" => return std::result::Result::Err("Parsing a container in this style is not supported in StateUpdates".to_string()),
                    "down_substates" => return std::result::Result::Err("Parsing a container in this style is not supported in StateUpdates".to_string()),
                    "new_roots" => return std::result::Result::Err("Parsing a container in this style is not supported in StateUpdates".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing StateUpdates".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(StateUpdates {
            down_virtual_substates: intermediate_rep.down_virtual_substates.into_iter().next().ok_or("down_virtual_substates missing in StateUpdates".to_string())?,
            up_substates: intermediate_rep.up_substates.into_iter().next().ok_or("up_substates missing in StateUpdates".to_string())?,
            down_substates: intermediate_rep.down_substates.into_iter().next().ok_or("down_substates missing in StateUpdates".to_string())?,
            new_roots: intermediate_rep.new_roots.into_iter().next().ok_or("new_roots missing in StateUpdates".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<StateUpdates> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<StateUpdates>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<StateUpdates>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for StateUpdates - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<StateUpdates> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <StateUpdates as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into StateUpdates - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


/// SBOR-encoded and then hex-encoded substate ID.
#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct SubstateId(String);

impl std::convert::From<String> for SubstateId {
    fn from(x: String) -> Self {
        SubstateId(x)
    }
}

impl std::string::ToString for SubstateId {
    fn to_string(&self) -> String {
       self.0.to_string()
    }
}

impl std::str::FromStr for SubstateId {
    type Err = std::string::ParseError;
    fn from_str(x: &str) -> std::result::Result<Self, Self::Err> {
        std::result::Result::Ok(SubstateId(x.to_string()))
    }
}

impl std::convert::From<SubstateId> for String {
    fn from(x: SubstateId) -> Self {
        x.0
    }
}

impl std::ops::Deref for SubstateId {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl std::ops::DerefMut for SubstateId {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct TransactionHeader {
    #[serde(rename = "version")]
    pub version: isize,

    #[serde(rename = "network_id")]
    pub network_id: isize,

    /// A decimal 64-bit unsigned integer.
    #[serde(rename = "start_epoch_inclusive")]
    pub start_epoch_inclusive: String,

    /// A decimal 64-bit unsigned integer.
    #[serde(rename = "end_epoch_exclusive")]
    pub end_epoch_exclusive: String,

    /// A decimal 64-bit unsigned integer.
    #[serde(rename = "nonce")]
    pub nonce: String,

    /// A hex-encoded public key of a notary.
    #[serde(rename = "notary_public_key")]
    pub notary_public_key: String,

    /// Specifies whether the notary's signature should be included in transaction signers list
    #[serde(rename = "notary_as_signatory")]
    pub notary_as_signatory: bool,

    /// Maximum number of cost units available for transaction execution. A decimal 32-bit unsigned integer.
    #[serde(rename = "cost_unit_limit")]
    pub cost_unit_limit: String,

    /// Specifies the validator tip. A decimal 32-bit unsigned integer, representing the percentage amount (a value of \"1\" corresponds to 1%).
    #[serde(rename = "tip_percentage")]
    pub tip_percentage: String,

}

impl TransactionHeader {
    pub fn new(version: isize, network_id: isize, start_epoch_inclusive: String, end_epoch_exclusive: String, nonce: String, notary_public_key: String, notary_as_signatory: bool, cost_unit_limit: String, tip_percentage: String, ) -> TransactionHeader {
        TransactionHeader {
            version: version,
            network_id: network_id,
            start_epoch_inclusive: start_epoch_inclusive,
            end_epoch_exclusive: end_epoch_exclusive,
            nonce: nonce,
            notary_public_key: notary_public_key,
            notary_as_signatory: notary_as_signatory,
            cost_unit_limit: cost_unit_limit,
            tip_percentage: tip_percentage,
        }
    }
}

/// Converts the TransactionHeader value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TransactionHeader {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("version".to_string());
        params.push(self.version.to_string());


        params.push("network_id".to_string());
        params.push(self.network_id.to_string());


        params.push("start_epoch_inclusive".to_string());
        params.push(self.start_epoch_inclusive.to_string());


        params.push("end_epoch_exclusive".to_string());
        params.push(self.end_epoch_exclusive.to_string());


        params.push("nonce".to_string());
        params.push(self.nonce.to_string());


        params.push("notary_public_key".to_string());
        params.push(self.notary_public_key.to_string());


        params.push("notary_as_signatory".to_string());
        params.push(self.notary_as_signatory.to_string());


        params.push("cost_unit_limit".to_string());
        params.push(self.cost_unit_limit.to_string());


        params.push("tip_percentage".to_string());
        params.push(self.tip_percentage.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TransactionHeader value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TransactionHeader {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub version: Vec<isize>,
            pub network_id: Vec<isize>,
            pub start_epoch_inclusive: Vec<String>,
            pub end_epoch_exclusive: Vec<String>,
            pub nonce: Vec<String>,
            pub notary_public_key: Vec<String>,
            pub notary_as_signatory: Vec<bool>,
            pub cost_unit_limit: Vec<String>,
            pub tip_percentage: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing TransactionHeader".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "version" => intermediate_rep.version.push(<isize as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "network_id" => intermediate_rep.network_id.push(<isize as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "start_epoch_inclusive" => intermediate_rep.start_epoch_inclusive.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "end_epoch_exclusive" => intermediate_rep.end_epoch_exclusive.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "nonce" => intermediate_rep.nonce.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "notary_public_key" => intermediate_rep.notary_public_key.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "notary_as_signatory" => intermediate_rep.notary_as_signatory.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "cost_unit_limit" => intermediate_rep.cost_unit_limit.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "tip_percentage" => intermediate_rep.tip_percentage.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing TransactionHeader".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TransactionHeader {
            version: intermediate_rep.version.into_iter().next().ok_or("version missing in TransactionHeader".to_string())?,
            network_id: intermediate_rep.network_id.into_iter().next().ok_or("network_id missing in TransactionHeader".to_string())?,
            start_epoch_inclusive: intermediate_rep.start_epoch_inclusive.into_iter().next().ok_or("start_epoch_inclusive missing in TransactionHeader".to_string())?,
            end_epoch_exclusive: intermediate_rep.end_epoch_exclusive.into_iter().next().ok_or("end_epoch_exclusive missing in TransactionHeader".to_string())?,
            nonce: intermediate_rep.nonce.into_iter().next().ok_or("nonce missing in TransactionHeader".to_string())?,
            notary_public_key: intermediate_rep.notary_public_key.into_iter().next().ok_or("notary_public_key missing in TransactionHeader".to_string())?,
            notary_as_signatory: intermediate_rep.notary_as_signatory.into_iter().next().ok_or("notary_as_signatory missing in TransactionHeader".to_string())?,
            cost_unit_limit: intermediate_rep.cost_unit_limit.into_iter().next().ok_or("cost_unit_limit missing in TransactionHeader".to_string())?,
            tip_percentage: intermediate_rep.tip_percentage.into_iter().next().ok_or("tip_percentage missing in TransactionHeader".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TransactionHeader> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<TransactionHeader>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<TransactionHeader>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for TransactionHeader - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<TransactionHeader> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <TransactionHeader as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into TransactionHeader - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct TransactionIntent {
    /// Transaction intent hash, hex-encoded.
    #[serde(rename = "hash")]
    pub hash: String,

    #[serde(rename = "header")]
    pub header: models::TransactionHeader,

    /// Transaction manifest, SBOR-encoded and then hex-encoded.
    #[serde(rename = "manifest")]
    pub manifest: String,

}

impl TransactionIntent {
    pub fn new(hash: String, header: models::TransactionHeader, manifest: String, ) -> TransactionIntent {
        TransactionIntent {
            hash: hash,
            header: header,
            manifest: manifest,
        }
    }
}

/// Converts the TransactionIntent value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TransactionIntent {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("hash".to_string());
        params.push(self.hash.to_string());

        // Skipping header in query parameter serialization


        params.push("manifest".to_string());
        params.push(self.manifest.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TransactionIntent value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TransactionIntent {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub hash: Vec<String>,
            pub header: Vec<models::TransactionHeader>,
            pub manifest: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing TransactionIntent".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "hash" => intermediate_rep.hash.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "header" => intermediate_rep.header.push(<models::TransactionHeader as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "manifest" => intermediate_rep.manifest.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing TransactionIntent".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TransactionIntent {
            hash: intermediate_rep.hash.into_iter().next().ok_or("hash missing in TransactionIntent".to_string())?,
            header: intermediate_rep.header.into_iter().next().ok_or("header missing in TransactionIntent".to_string())?,
            manifest: intermediate_rep.manifest.into_iter().next().ok_or("manifest missing in TransactionIntent".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TransactionIntent> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<TransactionIntent>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<TransactionIntent>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for TransactionIntent - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<TransactionIntent> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <TransactionIntent as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into TransactionIntent - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct TransactionPreviewRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: models::NetworkIdentifier,

    /// A transaction manifest. Sbor encoded, and then hex encoded.
    #[serde(rename = "manifest")]
    pub manifest: String,

    /// Maximum number of cost units available for transaction execution. A decimal 32-bit unsigned integer.
    #[serde(rename = "cost_unit_limit")]
    pub cost_unit_limit: String,

    /// The validator tip. A decimal 32-bit unsigned integer, representing the percentage amount (a value of \"1\" corresponds to 1%).
    #[serde(rename = "tip_percentage")]
    pub tip_percentage: String,

    /// The nonce value to use for execution. A decimal 64-bit unsigned integer.
    #[serde(rename = "nonce")]
    pub nonce: String,

    /// A list of public keys to be used as transaction signers, in a compressed format, hex encoded.
    #[serde(rename = "signer_public_keys")]
    pub signer_public_keys: Vec<String>,

    #[serde(rename = "flags")]
    pub flags: models::TransactionPreviewRequestFlags,

}

impl TransactionPreviewRequest {
    pub fn new(network_identifier: models::NetworkIdentifier, manifest: String, cost_unit_limit: String, tip_percentage: String, nonce: String, signer_public_keys: Vec<String>, flags: models::TransactionPreviewRequestFlags, ) -> TransactionPreviewRequest {
        TransactionPreviewRequest {
            network_identifier: network_identifier,
            manifest: manifest,
            cost_unit_limit: cost_unit_limit,
            tip_percentage: tip_percentage,
            nonce: nonce,
            signer_public_keys: signer_public_keys,
            flags: flags,
        }
    }
}

/// Converts the TransactionPreviewRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TransactionPreviewRequest {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping network_identifier in query parameter serialization


        params.push("manifest".to_string());
        params.push(self.manifest.to_string());


        params.push("cost_unit_limit".to_string());
        params.push(self.cost_unit_limit.to_string());


        params.push("tip_percentage".to_string());
        params.push(self.tip_percentage.to_string());


        params.push("nonce".to_string());
        params.push(self.nonce.to_string());


        params.push("signer_public_keys".to_string());
        params.push(self.signer_public_keys.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",").to_string());

        // Skipping flags in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TransactionPreviewRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TransactionPreviewRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub network_identifier: Vec<models::NetworkIdentifier>,
            pub manifest: Vec<String>,
            pub cost_unit_limit: Vec<String>,
            pub tip_percentage: Vec<String>,
            pub nonce: Vec<String>,
            pub signer_public_keys: Vec<Vec<String>>,
            pub flags: Vec<models::TransactionPreviewRequestFlags>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing TransactionPreviewRequest".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "network_identifier" => intermediate_rep.network_identifier.push(<models::NetworkIdentifier as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "manifest" => intermediate_rep.manifest.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "cost_unit_limit" => intermediate_rep.cost_unit_limit.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "tip_percentage" => intermediate_rep.tip_percentage.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "nonce" => intermediate_rep.nonce.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "signer_public_keys" => return std::result::Result::Err("Parsing a container in this style is not supported in TransactionPreviewRequest".to_string()),
                    "flags" => intermediate_rep.flags.push(<models::TransactionPreviewRequestFlags as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing TransactionPreviewRequest".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TransactionPreviewRequest {
            network_identifier: intermediate_rep.network_identifier.into_iter().next().ok_or("network_identifier missing in TransactionPreviewRequest".to_string())?,
            manifest: intermediate_rep.manifest.into_iter().next().ok_or("manifest missing in TransactionPreviewRequest".to_string())?,
            cost_unit_limit: intermediate_rep.cost_unit_limit.into_iter().next().ok_or("cost_unit_limit missing in TransactionPreviewRequest".to_string())?,
            tip_percentage: intermediate_rep.tip_percentage.into_iter().next().ok_or("tip_percentage missing in TransactionPreviewRequest".to_string())?,
            nonce: intermediate_rep.nonce.into_iter().next().ok_or("nonce missing in TransactionPreviewRequest".to_string())?,
            signer_public_keys: intermediate_rep.signer_public_keys.into_iter().next().ok_or("signer_public_keys missing in TransactionPreviewRequest".to_string())?,
            flags: intermediate_rep.flags.into_iter().next().ok_or("flags missing in TransactionPreviewRequest".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TransactionPreviewRequest> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<TransactionPreviewRequest>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<TransactionPreviewRequest>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for TransactionPreviewRequest - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<TransactionPreviewRequest> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <TransactionPreviewRequest as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into TransactionPreviewRequest - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct TransactionPreviewRequestFlags {
    #[serde(rename = "unlimited_loan")]
    pub unlimited_loan: bool,

}

impl TransactionPreviewRequestFlags {
    pub fn new(unlimited_loan: bool, ) -> TransactionPreviewRequestFlags {
        TransactionPreviewRequestFlags {
            unlimited_loan: unlimited_loan,
        }
    }
}

/// Converts the TransactionPreviewRequestFlags value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TransactionPreviewRequestFlags {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("unlimited_loan".to_string());
        params.push(self.unlimited_loan.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TransactionPreviewRequestFlags value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TransactionPreviewRequestFlags {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub unlimited_loan: Vec<bool>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing TransactionPreviewRequestFlags".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "unlimited_loan" => intermediate_rep.unlimited_loan.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing TransactionPreviewRequestFlags".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TransactionPreviewRequestFlags {
            unlimited_loan: intermediate_rep.unlimited_loan.into_iter().next().ok_or("unlimited_loan missing in TransactionPreviewRequestFlags".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TransactionPreviewRequestFlags> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<TransactionPreviewRequestFlags>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<TransactionPreviewRequestFlags>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for TransactionPreviewRequestFlags - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<TransactionPreviewRequestFlags> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <TransactionPreviewRequestFlags as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into TransactionPreviewRequestFlags - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct TransactionPreviewResponse {
    #[serde(rename = "receipt")]
    pub receipt: models::TransactionReceipt,

    #[serde(rename = "resource_changes")]
    pub resource_changes: Vec<models::ResourceChange>,

    #[serde(rename = "logs")]
    pub logs: Vec<models::TransactionPreviewResponseLogsInner>,

}

impl TransactionPreviewResponse {
    pub fn new(receipt: models::TransactionReceipt, resource_changes: Vec<models::ResourceChange>, logs: Vec<models::TransactionPreviewResponseLogsInner>, ) -> TransactionPreviewResponse {
        TransactionPreviewResponse {
            receipt: receipt,
            resource_changes: resource_changes,
            logs: logs,
        }
    }
}

/// Converts the TransactionPreviewResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TransactionPreviewResponse {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping receipt in query parameter serialization

        // Skipping resource_changes in query parameter serialization

        // Skipping logs in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TransactionPreviewResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TransactionPreviewResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub receipt: Vec<models::TransactionReceipt>,
            pub resource_changes: Vec<Vec<models::ResourceChange>>,
            pub logs: Vec<Vec<models::TransactionPreviewResponseLogsInner>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing TransactionPreviewResponse".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "receipt" => intermediate_rep.receipt.push(<models::TransactionReceipt as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "resource_changes" => return std::result::Result::Err("Parsing a container in this style is not supported in TransactionPreviewResponse".to_string()),
                    "logs" => return std::result::Result::Err("Parsing a container in this style is not supported in TransactionPreviewResponse".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing TransactionPreviewResponse".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TransactionPreviewResponse {
            receipt: intermediate_rep.receipt.into_iter().next().ok_or("receipt missing in TransactionPreviewResponse".to_string())?,
            resource_changes: intermediate_rep.resource_changes.into_iter().next().ok_or("resource_changes missing in TransactionPreviewResponse".to_string())?,
            logs: intermediate_rep.logs.into_iter().next().ok_or("logs missing in TransactionPreviewResponse".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TransactionPreviewResponse> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<TransactionPreviewResponse>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<TransactionPreviewResponse>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for TransactionPreviewResponse - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<TransactionPreviewResponse> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <TransactionPreviewResponse as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into TransactionPreviewResponse - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct TransactionPreviewResponseLogsInner {
    #[serde(rename = "level")]
    pub level: String,

    #[serde(rename = "message")]
    pub message: String,

}

impl TransactionPreviewResponseLogsInner {
    pub fn new(level: String, message: String, ) -> TransactionPreviewResponseLogsInner {
        TransactionPreviewResponseLogsInner {
            level: level,
            message: message,
        }
    }
}

/// Converts the TransactionPreviewResponseLogsInner value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TransactionPreviewResponseLogsInner {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("level".to_string());
        params.push(self.level.to_string());


        params.push("message".to_string());
        params.push(self.message.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TransactionPreviewResponseLogsInner value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TransactionPreviewResponseLogsInner {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub level: Vec<String>,
            pub message: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing TransactionPreviewResponseLogsInner".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "level" => intermediate_rep.level.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "message" => intermediate_rep.message.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing TransactionPreviewResponseLogsInner".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TransactionPreviewResponseLogsInner {
            level: intermediate_rep.level.into_iter().next().ok_or("level missing in TransactionPreviewResponseLogsInner".to_string())?,
            message: intermediate_rep.message.into_iter().next().ok_or("message missing in TransactionPreviewResponseLogsInner".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TransactionPreviewResponseLogsInner> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<TransactionPreviewResponseLogsInner>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<TransactionPreviewResponseLogsInner>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for TransactionPreviewResponseLogsInner - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<TransactionPreviewResponseLogsInner> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <TransactionPreviewResponseLogsInner as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into TransactionPreviewResponseLogsInner - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


/// The transaction execution receipt
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct TransactionReceipt {
    #[serde(rename = "status")]
    pub status: models::TransactionStatus,

    #[serde(rename = "fee_summary")]
    pub fee_summary: models::FeeSummary,

    #[serde(rename = "state_updates")]
    pub state_updates: models::StateUpdates,

    /// A list of new package addresses.
    #[serde(rename = "new_package_addresses")]
    pub new_package_addresses: Vec<models::PackageAddress>,

    /// A list of new component addresses.
    #[serde(rename = "new_component_addresses")]
    pub new_component_addresses: Vec<models::ComponentAddress>,

    /// A list of new resource addresses.
    #[serde(rename = "new_resource_addresses")]
    pub new_resource_addresses: Vec<models::ResourceAddress>,

    /// The engine return data (only present if status is succeeded)
    #[serde(rename = "output")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub output: Option<Vec<String>>,

    /// Error message (only present if status is failed or rejected)
    #[serde(rename = "error_message")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub error_message: Option<String>,

}

impl TransactionReceipt {
    pub fn new(status: models::TransactionStatus, fee_summary: models::FeeSummary, state_updates: models::StateUpdates, new_package_addresses: Vec<models::PackageAddress>, new_component_addresses: Vec<models::ComponentAddress>, new_resource_addresses: Vec<models::ResourceAddress>, ) -> TransactionReceipt {
        TransactionReceipt {
            status: status,
            fee_summary: fee_summary,
            state_updates: state_updates,
            new_package_addresses: new_package_addresses,
            new_component_addresses: new_component_addresses,
            new_resource_addresses: new_resource_addresses,
            output: None,
            error_message: None,
        }
    }
}

/// Converts the TransactionReceipt value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TransactionReceipt {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping status in query parameter serialization

        // Skipping fee_summary in query parameter serialization

        // Skipping state_updates in query parameter serialization


        params.push("new_package_addresses".to_string());
        params.push(self.new_package_addresses.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",").to_string());


        params.push("new_component_addresses".to_string());
        params.push(self.new_component_addresses.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",").to_string());


        params.push("new_resource_addresses".to_string());
        params.push(self.new_resource_addresses.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",").to_string());


        if let Some(ref output) = self.output {
            params.push("output".to_string());
            params.push(output.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",").to_string());
        }


        if let Some(ref error_message) = self.error_message {
            params.push("error_message".to_string());
            params.push(error_message.to_string());
        }

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TransactionReceipt value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TransactionReceipt {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub status: Vec<models::TransactionStatus>,
            pub fee_summary: Vec<models::FeeSummary>,
            pub state_updates: Vec<models::StateUpdates>,
            pub new_package_addresses: Vec<Vec<models::PackageAddress>>,
            pub new_component_addresses: Vec<Vec<models::ComponentAddress>>,
            pub new_resource_addresses: Vec<Vec<models::ResourceAddress>>,
            pub output: Vec<Vec<String>>,
            pub error_message: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing TransactionReceipt".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "status" => intermediate_rep.status.push(<models::TransactionStatus as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "fee_summary" => intermediate_rep.fee_summary.push(<models::FeeSummary as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "state_updates" => intermediate_rep.state_updates.push(<models::StateUpdates as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "new_package_addresses" => return std::result::Result::Err("Parsing a container in this style is not supported in TransactionReceipt".to_string()),
                    "new_component_addresses" => return std::result::Result::Err("Parsing a container in this style is not supported in TransactionReceipt".to_string()),
                    "new_resource_addresses" => return std::result::Result::Err("Parsing a container in this style is not supported in TransactionReceipt".to_string()),
                    "output" => return std::result::Result::Err("Parsing a container in this style is not supported in TransactionReceipt".to_string()),
                    "error_message" => intermediate_rep.error_message.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing TransactionReceipt".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TransactionReceipt {
            status: intermediate_rep.status.into_iter().next().ok_or("status missing in TransactionReceipt".to_string())?,
            fee_summary: intermediate_rep.fee_summary.into_iter().next().ok_or("fee_summary missing in TransactionReceipt".to_string())?,
            state_updates: intermediate_rep.state_updates.into_iter().next().ok_or("state_updates missing in TransactionReceipt".to_string())?,
            new_package_addresses: intermediate_rep.new_package_addresses.into_iter().next().ok_or("new_package_addresses missing in TransactionReceipt".to_string())?,
            new_component_addresses: intermediate_rep.new_component_addresses.into_iter().next().ok_or("new_component_addresses missing in TransactionReceipt".to_string())?,
            new_resource_addresses: intermediate_rep.new_resource_addresses.into_iter().next().ok_or("new_resource_addresses missing in TransactionReceipt".to_string())?,
            output: intermediate_rep.output.into_iter().next(),
            error_message: intermediate_rep.error_message.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TransactionReceipt> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<TransactionReceipt>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<TransactionReceipt>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for TransactionReceipt - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<TransactionReceipt> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <TransactionReceipt as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into TransactionReceipt - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


/// The status of the transaction
/// Enumeration of values.
/// Since this enum's variants do not hold data, we can easily define them them as `#[repr(C)]`
/// which helps with FFI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk_enum_derive::LabelledGenericEnum))]
pub enum TransactionStatus {
    #[serde(rename = "succeeded")]
    SUCCEEDED,
    #[serde(rename = "failed")]
    FAILED,
    #[serde(rename = "rejected")]
    REJECTED,
}

impl std::fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            TransactionStatus::SUCCEEDED => write!(f, "{}", "succeeded"),
            TransactionStatus::FAILED => write!(f, "{}", "failed"),
            TransactionStatus::REJECTED => write!(f, "{}", "rejected"),
        }
    }
}

impl std::str::FromStr for TransactionStatus {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "succeeded" => std::result::Result::Ok(TransactionStatus::SUCCEEDED),
            "failed" => std::result::Result::Ok(TransactionStatus::FAILED),
            "rejected" => std::result::Result::Ok(TransactionStatus::REJECTED),
            _ => std::result::Result::Err(format!("Value not valid: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct TransactionSubmitRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: models::NetworkIdentifier,

    /// A notarized transaction encoded in the Radix transaction format, and then hex encoded.
    #[serde(rename = "notarized_transaction")]
    pub notarized_transaction: String,

}

impl TransactionSubmitRequest {
    pub fn new(network_identifier: models::NetworkIdentifier, notarized_transaction: String, ) -> TransactionSubmitRequest {
        TransactionSubmitRequest {
            network_identifier: network_identifier,
            notarized_transaction: notarized_transaction,
        }
    }
}

/// Converts the TransactionSubmitRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TransactionSubmitRequest {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping network_identifier in query parameter serialization


        params.push("notarized_transaction".to_string());
        params.push(self.notarized_transaction.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TransactionSubmitRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TransactionSubmitRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub network_identifier: Vec<models::NetworkIdentifier>,
            pub notarized_transaction: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing TransactionSubmitRequest".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "network_identifier" => intermediate_rep.network_identifier.push(<models::NetworkIdentifier as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "notarized_transaction" => intermediate_rep.notarized_transaction.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing TransactionSubmitRequest".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TransactionSubmitRequest {
            network_identifier: intermediate_rep.network_identifier.into_iter().next().ok_or("network_identifier missing in TransactionSubmitRequest".to_string())?,
            notarized_transaction: intermediate_rep.notarized_transaction.into_iter().next().ok_or("notarized_transaction missing in TransactionSubmitRequest".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TransactionSubmitRequest> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<TransactionSubmitRequest>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<TransactionSubmitRequest>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for TransactionSubmitRequest - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<TransactionSubmitRequest> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <TransactionSubmitRequest as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into TransactionSubmitRequest - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct TransactionSubmitResponse {
    /// Is true if the transaction is a duplicate of an existing transaction in the mempool.
    #[serde(rename = "duplicate")]
    pub duplicate: bool,

}

impl TransactionSubmitResponse {
    pub fn new(duplicate: bool, ) -> TransactionSubmitResponse {
        TransactionSubmitResponse {
            duplicate: duplicate,
        }
    }
}

/// Converts the TransactionSubmitResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TransactionSubmitResponse {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("duplicate".to_string());
        params.push(self.duplicate.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TransactionSubmitResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TransactionSubmitResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub duplicate: Vec<bool>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing TransactionSubmitResponse".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "duplicate" => intermediate_rep.duplicate.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing TransactionSubmitResponse".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TransactionSubmitResponse {
            duplicate: intermediate_rep.duplicate.into_iter().next().ok_or("duplicate missing in TransactionSubmitResponse".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TransactionSubmitResponse> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<TransactionSubmitResponse>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<TransactionSubmitResponse>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for TransactionSubmitResponse - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<TransactionSubmitResponse> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <TransactionSubmitResponse as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into TransactionSubmitResponse - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct UpSubstate {
    /// SBOR-encoded and then hex-encoded substate ID.
    #[serde(rename = "substate_id")]
    pub substate_id: String,

    /// A decimal 32-bit unsigned integer
    #[serde(rename = "version")]
    pub version: String,

    /// SBOR-encoded and then hex-encoded substate bytes.
    #[serde(rename = "substate_bytes")]
    pub substate_bytes: String,

    /// JSON-encoded (and then stringified) substate model.
    #[serde(rename = "substate")]
    pub substate: String,

}

impl UpSubstate {
    pub fn new(substate_id: String, version: String, substate_bytes: String, substate: String, ) -> UpSubstate {
        UpSubstate {
            substate_id: substate_id,
            version: version,
            substate_bytes: substate_bytes,
            substate: substate,
        }
    }
}

/// Converts the UpSubstate value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for UpSubstate {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("substate_id".to_string());
        params.push(self.substate_id.to_string());


        params.push("version".to_string());
        params.push(self.version.to_string());


        params.push("substate_bytes".to_string());
        params.push(self.substate_bytes.to_string());


        params.push("substate".to_string());
        params.push(self.substate.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a UpSubstate value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for UpSubstate {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub substate_id: Vec<String>,
            pub version: Vec<String>,
            pub substate_bytes: Vec<String>,
            pub substate: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing UpSubstate".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "substate_id" => intermediate_rep.substate_id.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "version" => intermediate_rep.version.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "substate_bytes" => intermediate_rep.substate_bytes.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "substate" => intermediate_rep.substate.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing UpSubstate".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(UpSubstate {
            substate_id: intermediate_rep.substate_id.into_iter().next().ok_or("substate_id missing in UpSubstate".to_string())?,
            version: intermediate_rep.version.into_iter().next().ok_or("version missing in UpSubstate".to_string())?,
            substate_bytes: intermediate_rep.substate_bytes.into_iter().next().ok_or("substate_bytes missing in UpSubstate".to_string())?,
            substate: intermediate_rep.substate.into_iter().next().ok_or("substate missing in UpSubstate".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<UpSubstate> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<UpSubstate>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<UpSubstate>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for UpSubstate - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<UpSubstate> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <UpSubstate as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into UpSubstate - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


/// SBOR-encoded and then hex-encoded virtual substate ID.
#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct VirtualSubstateId(String);

impl std::convert::From<String> for VirtualSubstateId {
    fn from(x: String) -> Self {
        VirtualSubstateId(x)
    }
}

impl std::string::ToString for VirtualSubstateId {
    fn to_string(&self) -> String {
       self.0.to_string()
    }
}

impl std::str::FromStr for VirtualSubstateId {
    type Err = std::string::ParseError;
    fn from_str(x: &str) -> std::result::Result<Self, Self::Err> {
        std::result::Result::Ok(VirtualSubstateId(x.to_string()))
    }
}

impl std::convert::From<VirtualSubstateId> for String {
    fn from(x: VirtualSubstateId) -> Self {
        x.0
    }
}

impl std::ops::Deref for VirtualSubstateId {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl std::ops::DerefMut for VirtualSubstateId {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

