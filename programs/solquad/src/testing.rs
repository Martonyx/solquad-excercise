use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

entrypoint!(process_instruction);

// Define the instruction data structure
#[derive(Debug)]
pub enum TokenInstruction {
    // Initialize the token with the specified total supply
    Initialize { total_supply: u64 },
    // Transfer tokens from the sender to the specified recipient
    Transfer { amount: u64 },
    // Get the token balance of the specified account
    GetBalance,
    // Approve a spender to spend tokens on behalf of the sender
    Approve { spender: Pubkey, amount: u64 },
}

// Define the token state
pub struct Token {
    pub total_supply: u64,
    pub owner: Pubkey,
    pub balances: Vec<(Pubkey, u64)>,
    pub allowances: Vec<(Pubkey, Pubkey, u64)>,
}

impl Token {
    // Initialize a new token
    pub fn initialize(&mut self, total_supply: u64, owner: Pubkey) {
        self.total_supply = total_supply;
        self.owner = owner;
        self.balances.push((owner, total_supply));
    }

    // Transfer tokens from sender to recipient
    pub fn transfer(&mut self, sender: &Pubkey, recipient: &Pubkey, amount: u64) -> ProgramResult {
        let mut sender_index = None;
        let mut recipient_index = None;

        for (i, (account, balance)) in self.balances.iter_mut().enumerate() {
            if *account == *sender {
                sender_index = Some(i);
            }
            if *account == *recipient {
                recipient_index = Some(i);
            }
            if sender_index.is_some() && recipient_index.is_some() {
                break;
            }
        }

        let sender_index = sender_index.ok_or(ProgramError::InvalidArgument)?;
        let recipient_index = recipient_index.ok_or(ProgramError::InvalidArgument)?;

        if self.balances[sender_index].1 < amount {
            return Err(ProgramError::InsufficientFunds);
        }

        self.balances[sender_index].1 -= amount;
        self.balances[recipient_index].1 += amount;

        Ok(())
    }

    // Get the token balance of an account
    pub fn get_balance(&self, account: &Pubkey) -> Option<u64> {
        self.balances
            .iter()
            .find(|(acc, _)| *acc == *account)
            .map(|(_, balance)| *balance)
    }

    // Approve a spender to spend tokens on behalf of the sender
    pub fn approve(&mut self, sender: &Pubkey, spender: &Pubkey, amount: u64) -> ProgramResult {
        let allowance_index = self
            .allowances
            .iter_mut()
            .position(|(owner, spender_account, _)| *owner == *sender && *spender_account == *spender);

        match allowance_index {
            Some(index) => {
                self.allowances[index].2 = amount;
            }
            None => {
                self.allowances.push((*sender, *spender, amount));
            }
        }

        Ok(())
    }
}

// Process instructions
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Match the instruction data to call corresponding functions
    let instruction = TokenInstruction::unpack(instruction_data)?;

    match instruction {
        TokenInstruction::Initialize { total_supply } => {
            let mut token = Token {
                total_supply: 0,
                owner: *accounts[0].key,
                balances: vec![],
                allowances: vec![],
            };
            token.initialize(total_supply, *accounts[0].key);
            Ok(())
        }
        TokenInstruction::Transfer { amount } => {
            // Transfer tokens from sender to recipient
            let sender = next_account_info(accounts)?;
            let recipient = next_account_info(accounts)?;

            let mut token = Token {
                total_supply: 0,
                owner: *accounts[0].key,
                balances: vec![],
                allowances: vec![],
            };

            token.transfer(sender.key, recipient.key, amount)?;
            Ok(())
        }
        TokenInstruction::GetBalance => {
            // Get the token balance of an account
            let account = next_account_info(accounts)?;

            let token = Token {
                total_supply: 0,
                owner: *accounts[0].key,
                balances: vec![],
                allowances: vec![],
            };

            let balance = token.get_balance(account.key).unwrap_or(0);
            msg!("Account balance: {}", balance);
            Ok(())
        }
        TokenInstruction::Approve { spender, amount } => {
            // Approve a spender to spend tokens on behalf of the sender
            let owner = next_account_info(accounts)?;

            let mut token = Token {
                total_supply: 0,
                owner: *accounts[0].key,
                balances: vec![],
                allowances: vec![],
            };

            token.approve(owner.key, &spender, amount)?;
            Ok(())
        }
    }
}

impl TokenInstruction {
    // Unpack the instruction data
    fn unpack(data: &[u8]) -> Result<Self, ProgramError> {
        use ProgramError::InvalidInstruction;
        let (&tag, rest) = data.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => Self::Initialize {
                total_supply: Self::unpack_u64(rest)?,
            },
            1 => Self::Transfer {
                amount: Self::unpack_u64(rest)?,
            },
            2 => Self::GetBalance,
            3 => {
                let (spender, amount) = Self::unpack_approve(rest)?;
                Self::Approve { spender, amount }
            }
            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_u64(input: &[u8]) -> Result<u64, ProgramError> {
        if input.len() < 8 {
            return Err(ProgramError::InvalidInstruction);
        }
        let (bytes, _rest) = input.split_at(8);
        Ok(u64::from_le_bytes(
            bytes.try_into().expect("slice with incorrect length"),
        ))
    }

    fn unpack_approve(input: &[u8]) -> Result<(Pubkey, u64), ProgramError> {
        let (spender, rest) = Self::unpack_pubkey(input)?;
        let amount = Self::unpack_u64(rest)?;
        Ok((spender, amount))
    }

    fn unpack_pubkey(input: &[u8]) -> Result<(Pubkey, &[u8]), ProgramError> {
        use ProgramError::InvalidInstruction;
        let (key, rest) = input.split_at(32);
        Ok((Pubkey::new(key), rest))
    }
}