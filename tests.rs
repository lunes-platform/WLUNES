#[cfg(test)]
mod tests {
    use super::wlunes::*;
    use ink::env::{
        test::{self, DefaultAccounts, EmittedEvent},
        DefaultEnvironment,
    };

    type Event = <Wlunes as ink::reflect::ContractEventBase>::Type;

    fn default_accounts() -> DefaultAccounts<DefaultEnvironment> {
        test::default_accounts::<DefaultEnvironment>()
    }

    fn set_next_caller(caller: ink::primitives::AccountId) {
        test::set_caller::<DefaultEnvironment>(caller);
    }

    fn set_balance(account: ink::primitives::AccountId, balance: u128) {
        test::set_account_balance::<DefaultEnvironment>(account, balance);
    }

    fn get_balance(account: ink::primitives::AccountId) -> u128 {
        test::get_account_balance::<DefaultEnvironment>(account).unwrap_or(0)
    }

    fn contract_id() -> ink::primitives::AccountId {
        test::callee::<DefaultEnvironment>()
    }

    #[ink::test]
    fn constructor_works() {
        let wlunes = Wlunes::new();
        assert_eq!(wlunes.total_supply(), 0);
        assert_eq!(wlunes.token_name(), Some("Wrapped Lunes".into()));
        assert_eq!(wlunes.token_symbol(), Some("WLUNES".into()));
        assert_eq!(wlunes.token_decimals(), 8);
    }

    #[ink::test]
    fn deposit_works() {
        let mut wlunes = Wlunes::new();
        let accounts = default_accounts();
        
        // Set caller and give them some native balance
        set_next_caller(accounts.alice);
        set_balance(accounts.alice, 1000);
        
        // Set transferred value to simulate payable call
        test::set_value_transferred::<DefaultEnvironment>(100);
        
        // Deposit should work
        assert_eq!(wlunes.deposit(), Ok(()));
        assert_eq!(wlunes.total_supply(), 100);
        assert_eq!(wlunes.balance_of(accounts.alice), 100);
    }

    #[ink::test]
    fn deposit_zero_amount_fails() {
        let mut wlunes = Wlunes::new();
        let accounts = default_accounts();
        
        set_next_caller(accounts.alice);
        test::set_value_transferred::<DefaultEnvironment>(0);
        
        assert_eq!(wlunes.deposit(), Err(Error::ZeroAmount));
    }

    #[ink::test]
    fn withdraw_works() {
        let mut wlunes = Wlunes::new();
        let accounts = default_accounts();
        
        // First deposit some tokens
        set_next_caller(accounts.alice);
        set_balance(accounts.alice, 1000);
        test::set_value_transferred::<DefaultEnvironment>(100);
        assert_eq!(wlunes.deposit(), Ok(()));
        
        // Set contract balance for withdrawal
        set_balance(contract_id(), 100);
        
        // Now withdraw
        assert_eq!(wlunes.withdraw(50), Ok(()));
        assert_eq!(wlunes.total_supply(), 50);
        assert_eq!(wlunes.balance_of(accounts.alice), 50);
    }

    #[ink::test]
    fn withdraw_insufficient_balance_fails() {
        let mut wlunes = Wlunes::new();
        let accounts = default_accounts();
        
        set_next_caller(accounts.alice);
        assert_eq!(wlunes.withdraw(100), Err(Error::InsufficientBalance));
    }

    #[ink::test]
    fn withdraw_zero_amount_fails() {
        let mut wlunes = Wlunes::new();
        let accounts = default_accounts();
        
        set_next_caller(accounts.alice);
        assert_eq!(wlunes.withdraw(0), Err(Error::ZeroAmount));
    }

    #[ink::test]
    fn transfer_works() {
        let mut wlunes = Wlunes::new();
        let accounts = default_accounts();
        
        // Give Alice some tokens
        set_next_caller(accounts.alice);
        set_balance(accounts.alice, 1000);
        test::set_value_transferred::<DefaultEnvironment>(100);
        assert_eq!(wlunes.deposit(), Ok(()));
        
        // Transfer to Bob
        assert_eq!(wlunes.transfer(accounts.bob, 30, vec![]), Ok(()));
        assert_eq!(wlunes.balance_of(accounts.alice), 70);
        assert_eq!(wlunes.balance_of(accounts.bob), 30);
    }

    #[ink::test]
    fn transfer_insufficient_balance_fails() {
        let mut wlunes = Wlunes::new();
        let accounts = default_accounts();
        
        set_next_caller(accounts.alice);
        assert_eq!(wlunes.transfer(accounts.bob, 100, vec![]), Err(Error::InsufficientBalance));
    }

    #[ink::test]
    fn transfer_zero_amount_fails() {
        let mut wlunes = Wlunes::new();
        let accounts = default_accounts();
        
        set_next_caller(accounts.alice);
        assert_eq!(wlunes.transfer(accounts.bob, 0, vec![]), Err(Error::ZeroAmount));
    }

    #[ink::test]
    fn approve_works() {
        let mut wlunes = Wlunes::new();
        let accounts = default_accounts();
        
        set_next_caller(accounts.alice);
        assert_eq!(wlunes.approve(accounts.bob, 100), Ok(()));
        assert_eq!(wlunes.allowance(accounts.alice, accounts.bob), 100);
    }

    #[ink::test]
    fn transfer_from_works() {
        let mut wlunes = Wlunes::new();
        let accounts = default_accounts();
        
        // Give Alice some tokens
        set_next_caller(accounts.alice);
        set_balance(accounts.alice, 1000);
        test::set_value_transferred::<DefaultEnvironment>(100);
        assert_eq!(wlunes.deposit(), Ok(()));
        
        // Alice approves Bob to spend 50 tokens
        assert_eq!(wlunes.approve(accounts.bob, 50), Ok(()));
        
        // Bob transfers from Alice to Charlie
        set_next_caller(accounts.bob);
        assert_eq!(wlunes.transfer_from(accounts.alice, accounts.charlie, 30, vec![]), Ok(()));
        
        assert_eq!(wlunes.balance_of(accounts.alice), 70);
        assert_eq!(wlunes.balance_of(accounts.charlie), 30);
        assert_eq!(wlunes.allowance(accounts.alice, accounts.bob), 20);
    }

    #[ink::test]
    fn transfer_from_insufficient_allowance_fails() {
        let mut wlunes = Wlunes::new();
        let accounts = default_accounts();
        
        set_next_caller(accounts.bob);
        assert_eq!(
            wlunes.transfer_from(accounts.alice, accounts.charlie, 100, vec![]),
            Err(Error::InsufficientAllowance)
        );
    }

    #[ink::test]
    fn burn_works() {
        let mut wlunes = Wlunes::new();
        let accounts = default_accounts();
        
        // Give Alice some tokens
        set_next_caller(accounts.alice);
        set_balance(accounts.alice, 1000);
        test::set_value_transferred::<DefaultEnvironment>(100);
        assert_eq!(wlunes.deposit(), Ok(()));
        
        // Alice burns her own tokens
        assert_eq!(wlunes.burn(accounts.alice, 30), Ok(()));
        assert_eq!(wlunes.balance_of(accounts.alice), 70);
        assert_eq!(wlunes.total_supply(), 70);
    }

    #[ink::test]
    fn burn_with_allowance_works() {
        let mut wlunes = Wlunes::new();
        let accounts = default_accounts();
        
        // Give Alice some tokens
        set_next_caller(accounts.alice);
        set_balance(accounts.alice, 1000);
        test::set_value_transferred::<DefaultEnvironment>(100);
        assert_eq!(wlunes.deposit(), Ok(()));
        
        // Alice approves Bob to burn her tokens
        assert_eq!(wlunes.approve(accounts.bob, 50), Ok(()));
        
        // Bob burns Alice's tokens
        set_next_caller(accounts.bob);
        assert_eq!(wlunes.burn(accounts.alice, 30), Ok(()));
        
        assert_eq!(wlunes.balance_of(accounts.alice), 70);
        assert_eq!(wlunes.total_supply(), 70);
        assert_eq!(wlunes.allowance(accounts.alice, accounts.bob), 20);
    }

    #[ink::test]
    fn burn_insufficient_balance_fails() {
        let mut wlunes = Wlunes::new();
        let accounts = default_accounts();
        
        set_next_caller(accounts.alice);
        assert_eq!(wlunes.burn(accounts.alice, 100), Err(Error::InsufficientBalance));
    }

    #[ink::test]
    fn burn_insufficient_allowance_fails() {
        let mut wlunes = Wlunes::new();
        let accounts = default_accounts();
        
        // Give Alice some tokens
        set_next_caller(accounts.alice);
        set_balance(accounts.alice, 1000);
        test::set_value_transferred::<DefaultEnvironment>(100);
        assert_eq!(wlunes.deposit(), Ok(()));
        
        // Bob tries to burn Alice's tokens without allowance
        set_next_caller(accounts.bob);
        assert_eq!(wlunes.burn(accounts.alice, 30), Err(Error::InsufficientAllowance));
    }

    #[ink::test]
    fn invariant_total_supply_equals_contract_balance() {
        let mut wlunes = Wlunes::new();
        let accounts = default_accounts();
        
        // Initial state
        assert_eq!(wlunes.total_supply(), 0);
        
        // After deposit
        set_next_caller(accounts.alice);
        set_balance(accounts.alice, 1000);
        test::set_value_transferred::<DefaultEnvironment>(100);
        assert_eq!(wlunes.deposit(), Ok(()));
        
        // Invariant: total_supply should equal the amount deposited
        assert_eq!(wlunes.total_supply(), 100);
        
        // After another deposit
        set_next_caller(accounts.bob);
        set_balance(accounts.bob, 1000);
        test::set_value_transferred::<DefaultEnvironment>(50);
        assert_eq!(wlunes.deposit(), Ok(()));
        
        // Invariant: total_supply should equal total deposited
        assert_eq!(wlunes.total_supply(), 150);
        
        // After withdrawal
        set_next_caller(accounts.alice);
        set_balance(contract_id(), 150);
        assert_eq!(wlunes.withdraw(30), Ok(()));
        
        // Invariant: total_supply should decrease by withdrawn amount
        assert_eq!(wlunes.total_supply(), 120);
    }

    #[ink::test]
    fn events_are_emitted() {
        let mut wlunes = Wlunes::new();
        let accounts = default_accounts();
        
        // Test deposit event
        set_next_caller(accounts.alice);
        set_balance(accounts.alice, 1000);
        test::set_value_transferred::<DefaultEnvironment>(100);
        assert_eq!(wlunes.deposit(), Ok(()));
        
        let emitted_events = test::recorded_events().collect::<Vec<_>>();
        assert_eq!(emitted_events.len(), 1);
        
        // Test transfer event
        assert_eq!(wlunes.transfer(accounts.bob, 30, vec![]), Ok(()));
        
        let emitted_events = test::recorded_events().collect::<Vec<_>>();
        assert_eq!(emitted_events.len(), 2);
        
        // Test approval event
        assert_eq!(wlunes.approve(accounts.bob, 50), Ok(()));
        
        let emitted_events = test::recorded_events().collect::<Vec<_>>();
        assert_eq!(emitted_events.len(), 3);
    }

    #[ink::test]
    fn overflow_protection_works() {
        let mut wlunes = Wlunes::new();
        let accounts = default_accounts();
        
        // Test that we can't overflow total_supply
        set_next_caller(accounts.alice);
        set_balance(accounts.alice, u128::MAX);
        test::set_value_transferred::<DefaultEnvironment>(u128::MAX);
        
        // First deposit should work
        assert_eq!(wlunes.deposit(), Ok(()));
        
        // Second deposit should fail due to overflow
        test::set_value_transferred::<DefaultEnvironment>(1);
        assert_eq!(wlunes.deposit(), Err(Error::Overflow));
    }
}
