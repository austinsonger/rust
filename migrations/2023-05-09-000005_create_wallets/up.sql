CREATE TYPE wallet_type AS ENUM (
    'btc',
    'xmr'
);

CREATE TYPE transaction_type AS ENUM (
    'deposit',
    'withdrawal',
    'escrow_lock',
    'escrow_release',
    'fee'
);

CREATE TABLE wallets (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id),
    wallet_type wallet_type NOT NULL,
    encrypted_private_key TEXT NOT NULL,
    public_address VARCHAR(255) NOT NULL,
    balance DECIMAL(20, 12) NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT unique_user_wallet UNIQUE (user_id, wallet_type)
);

CREATE TABLE transactions (
    id SERIAL PRIMARY KEY,
    wallet_id INTEGER NOT NULL REFERENCES wallets(id),
    transaction_type transaction_type NOT NULL,
    amount DECIMAL(20, 12) NOT NULL,
    fee DECIMAL(20, 12) NOT NULL DEFAULT 0,
    tx_hash VARCHAR(255),
    order_id INTEGER REFERENCES orders(id),
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP
);

CREATE INDEX idx_wallets_user ON wallets(user_id);
CREATE INDEX idx_transactions_wallet ON transactions(wallet_id);
CREATE INDEX idx_transactions_order ON transactions(order_id);
