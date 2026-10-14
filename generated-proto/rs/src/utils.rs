use tracing::trace;

pub fn retrieve_active_bitfields<T: buffa::Enumeration>(bitfield: i64) -> impl Iterator<Item = T> {
    let mut copy = bitfield;

    std::iter::from_fn(move || {
        while copy != 0 {
            let index = copy.trailing_zeros();
            copy &= copy - 1;

            if let Some(role) = T::from_i32(index as i32) {
                return Some(role);
            } else {
                trace!(enum_type = %std::any::type_name::<T>(), "Unknown index {} in bitfield {}", index, bitfield);
            }
        }
        None
    })
}

pub fn compile_bitfield<T: buffa::Enumeration>(items: impl IntoIterator<Item = T>) -> i64 {
    items
        .into_iter()
        .fold(0, |acc, item| acc | (1 << T::to_i32(&item) as i64))
}

pub mod user_identifying {
    use connectrpc::ConnectError;

    use crate::proto::auth::v2::{Permission, SmallData, UserIdentifier, UserIdentifierView};

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum GenericUserIdentifier {
        UserId(i64),
        Username(String),
        DiscordID(String),
    }

    impl GenericUserIdentifier {
        pub fn to_user_identifier(self) -> UserIdentifier {
            use crate::proto::auth::v2::user_identifier::UserIdentifier as EnumUserIdentifier;
            match self {
                GenericUserIdentifier::UserId(id) => UserIdentifier {
                    user_identifier: Some(EnumUserIdentifier::UserId(id)),
                    ..Default::default()
                },
                GenericUserIdentifier::Username(username) => UserIdentifier {
                    user_identifier: Some(EnumUserIdentifier::Username(username)),
                    ..Default::default()
                },
                GenericUserIdentifier::DiscordID(id) => UserIdentifier {
                    user_identifier: Some(EnumUserIdentifier::DiscordId(id)),
                    ..Default::default()
                },
            }
        }
    }

    pub fn resolve_user_identifier_view(
        proto_user_identifier: Option<&UserIdentifierView>,
    ) -> Option<GenericUserIdentifier> {
        if let Some(user_identifier_) = proto_user_identifier
            && let Some(ref user_identifier) = user_identifier_.user_identifier
        {
            use crate::proto::auth::v2::user_identifier::UserIdentifierView;
            match user_identifier {
                UserIdentifierView::UserId(user_id) => {
                    Some(GenericUserIdentifier::UserId(*user_id))
                }
                UserIdentifierView::Username(username) => {
                    Some(GenericUserIdentifier::Username(username.to_string()))
                }
                UserIdentifierView::DiscordId(discord_id) => {
                    Some(GenericUserIdentifier::DiscordID(discord_id.to_string()))
                }
            }
        } else {
            None
        }
    }

    /// Resolves the user_identifier for this request
    /// If the user is an admin, they are allowed to override what user they are acting as by providing a user_identifier in the request.
    /// If the user is not an admin, they are not allowed to provide a user_identifier, and will only be able to act as themselves.
    pub fn resolve_overridable_user_identifier_from_message(
        user_identifier: &buffa::MessageFieldView<UserIdentifierView>,
        current_user: &SmallData,
    ) -> Result<GenericUserIdentifier, ConnectError> {
        if (current_user.permissions & (1 << Permission::PERMISSION_ADMIN as i64) != 0)
            && user_identifier.is_set()
        {
            resolve_user_identifier_view(user_identifier.as_option()).ok_or_else(|| {
                ConnectError::invalid_argument("user_identifier is set, yet invalid")
            })
        } else if user_identifier.is_set() {
            Err(ConnectError::permission_denied(
                "You must be an admin to set the user_identifier field in the request",
            ))
        } else {
            Ok(GenericUserIdentifier::UserId(current_user.user_id))
        }
    }
}
