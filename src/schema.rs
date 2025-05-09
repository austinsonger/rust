// @generated automatically by Diesel CLI.

diesel::table! {
    categories (id) {
        id -> Int4,
        name -> Varchar,
        description -> Nullable<Text>,
        parent_id -> Nullable<Int4>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    conversations (id) {
        id -> Int4,
        user1_id -> Int4,
        user2_id -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    messages (id) {
        id -> Int4,
        conversation_id -> Int4,
        sender_id -> Int4,
        encrypted_content -> Text,
        is_read -> Bool,
        created_at -> Timestamp,
    }
}

diesel::table! {
    order_items (id) {
        id -> Int4,
        order_id -> Int4,
        product_id -> Int4,
        variant_id -> Nullable<Int4>,
        quantity -> Int4,
        price_per_unit -> Numeric,
        created_at -> Timestamp,
    }
}

diesel::table! {
    order_status_history (id) {
        id -> Int4,
        order_id -> Int4,
        status -> crate::models::order::OrderStatusMapping,
        notes -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    orders (id) {
        id -> Int4,
        buyer_id -> Int4,
        vendor_id -> Int4,
        status -> crate::models::order::OrderStatusMapping,
        currency -> crate::models::payment::PaymentCurrencyMapping,
        total_amount -> Numeric,
        escrow_address -> Nullable<Varchar>,
        encrypted_shipping_address -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        completed_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    product_images (id) {
        id -> Int4,
        product_id -> Int4,
        image_data -> Bytea,
        is_primary -> Bool,
        created_at -> Timestamp,
    }
}

diesel::table! {
    product_variants (id) {
        id -> Int4,
        product_id -> Int4,
        title -> Varchar,
        description -> Nullable<Text>,
        price_btc -> Nullable<Numeric>,
        price_xmr -> Nullable<Numeric>,
        stock -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    products (id) {
        id -> Int4,
        vendor_id -> Int4,
        title -> Varchar,
        description -> Text,
        category_id -> Nullable<Int4>,
        price_btc -> Nullable<Numeric>,
        price_xmr -> Nullable<Numeric>,
        stock -> Int4,
        is_active -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    reviews (id) {
        id -> Int4,
        order_id -> Int4,
        reviewer_id -> Int4,
        vendor_id -> Int4,
        product_id -> Int4,
        rating -> Int4,
        comment -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    transactions (id) {
        id -> Int4,
        wallet_id -> Int4,
        transaction_type -> crate::models::payment::TransactionTypeMapping,
        amount -> Numeric,
        fee -> Numeric,
        tx_hash -> Nullable<Varchar>,
        order_id -> Nullable<Int4>,
        status -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        completed_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        password_hash -> Varchar,
        pgp_public_key -> Nullable<Text>,
        role -> Varchar,
        reputation -> Nullable<Float8>,
        is_locked -> Nullable<Bool>,
        locked_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    vendor_bonds (id) {
        id -> Int4,
        vendor_id -> Int4,
        amount_btc -> Numeric,
        status -> Varchar,
        transaction_id -> Nullable<Int4>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        approved_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    wallets (id) {
        id -> Int4,
        user_id -> Int4,
        wallet_type -> crate::models::payment::WalletTypeMapping,
        encrypted_private_key -> Text,
        public_address -> Varchar,
        balance -> Numeric,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(order_items -> orders (order_id));
diesel::joinable!(order_items -> product_variants (variant_id));
diesel::joinable!(order_items -> products (product_id));
diesel::joinable!(order_status_history -> orders (order_id));
diesel::joinable!(product_images -> products (product_id));
diesel::joinable!(product_variants -> products (product_id));
diesel::joinable!(products -> categories (category_id));
diesel::joinable!(reviews -> orders (order_id));
diesel::joinable!(reviews -> products (product_id));
diesel::joinable!(transactions -> orders (order_id));
diesel::joinable!(transactions -> wallets (wallet_id));
diesel::joinable!(vendor_bonds -> transactions (transaction_id));
diesel::joinable!(vendor_bonds -> users (vendor_id));
diesel::joinable!(wallets -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    categories,
    conversations,
    messages,
    order_items,
    order_status_history,
    orders,
    product_images,
    product_variants,
    products,
    reviews,
    transactions,
    users,
    vendor_bonds,
    wallets,
);
