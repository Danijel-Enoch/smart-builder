#[payable]
pub fn create_non_fungible_token(&mut self, args: TokenArgs) -> Promise {
    if env::attached_deposit() > 0 {
        self.storage_deposit();
    }
    args.metadata.assert_valid();
    let token_id = args.metadata.symbol.to_ascii_lowercase();
    assert!(is_valid_token_id(&token_id), "Invalid Symbol");
    let token_account_id = format!("{}.{}", token_id, env::current_account_id());
    assert!(
        env::is_valid_account_id(token_account_id.as_bytes()),
        "Token Account ID is invalid"
    );

    let account_id = env::predecessor_account_id();

    let required_balance = self.get_min_attached_balance(&args);
    let user_balance = self.storage_deposits.get(&account_id).unwrap_or(0);
    assert!(
        user_balance >= required_balance,
        "Not enough required balance"
    );
    self.storage_deposits
        .insert(&account_id, &(user_balance - required_balance));

    let initial_storage_usage = env::storage_usage();

    assert!(
        self.tokens.insert(&token_id, &args).is_none(),
        "Token ID is already taken"
    );

    let storage_balance_used =
        Balance::from(env::storage_usage() - initial_storage_usage) * STORAGE_PRICE_PER_BYTE;

    Promise::new(token_account_id)
        .create_account()
        .transfer(required_balance - storage_balance_used)
        .deploy_contract(FT_WASM_CODE.to_vec())
        .function_call(b"new".to_vec(), serde_json::to_vec(&args).unwrap(), 0, GAS)
}
