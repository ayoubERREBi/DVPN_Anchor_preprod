use anchor_lang::prelude::*;


declare_id!("6CkLPySkXkYgLMX1bVMoDoEAANjLEp4X9Sj3yHZK4K3q");

#[program]
pub mod dvpn {
    use super::*;

    pub fn create_node(ctx: Context<CreateNode>, wg_key: String, price_hour: u64) -> Result<()> {
        ctx.accounts.node.create(ctx.accounts.user.key(),wg_key,price_hour);

        Ok(())
    }

    pub fn create_client(ctx: Context<CreateClient>, wg_key: String) -> Result<()> {
        ctx.accounts.client.create(ctx.accounts.user.key(),wg_key);

        Ok(())
    }

    pub fn start_session(ctx : Context<StartSession>, session_id:String )->Result<()>{
        let session = &mut ctx.accounts.session;
        let node = &mut ctx.accounts.node;
        let client = &mut ctx.accounts.client;

        session.start(client.key(), node.key(), node.price_hour, session_id)?;
        
        // Mettre à jour l'état des comptes
        node.in_session = true;
        client.in_session = true;

        Ok(())
    }

    pub fn stop_session(ctx: Context<StopSession>) -> Result<()> {
        let session = &mut ctx.accounts.session;
        let node = &mut ctx.accounts.node;
        let client = &mut ctx.accounts.client;

        let now = Clock::get()?.unix_timestamp;
        session.end_time = now;

        // Réinitialiser le statut
        node.in_session = false;
        client.in_session = false;

        Ok(())
    }




    
}



#[derive(Accounts)]
pub struct CreateNode<'info>{
    #[account(init, payer=user, space = 8 + Node::INIT_SPACE, seeds = [b"node", user.key().as_ref()], bump)]
    pub node : Account<'info, Node>,

    #[account(mut)]
    pub user : Signer<'info>,

    pub system_program : Program<'info, System>,

}

#[derive(Accounts)]
pub struct CreateClient<'info>{
    #[account(init, payer=user, space = 8 + Client::INIT_SPACE, seeds = [b"client", user.key().as_ref()], bump)]
    pub client : Account<'info, Client>,

    #[account(mut)]
    pub user : Signer<'info>,

    pub system_program : Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(session_id: String)]
pub struct StartSession<'info>{
    #[account(init,
              payer = user,
              space = 8 + Session::INIT_SPACE,
              seeds = [b"session", user.key().as_ref(),session_id.as_bytes()],
            bump)]
    pub session : Account<'info,Session>,
    

    #[account(mut)]
    pub node: Account<'info, Node>,

    #[account(mut)]
    pub client: Account<'info, Client>,

    #[account(mut)]
    pub user : Signer<'info>,

    pub system_program : Program<'info, System>
}

#[derive(Accounts)]
pub struct StopSession<'info> {
    #[account(
        mut,
        constraint = session.end_time == 0 @ ErrorCode::SessionAlreadyEnded
    )]
    pub session: Account<'info, Session>,

    #[account(
        mut,
        constraint = session.client_key == client.key() @ ErrorCode::InvalidClient
    )]
    pub client: Account<'info, Client>,

    #[account(
        mut,
        constraint = session.node_key == node.key() @ ErrorCode::InvalidNode
    )]
    pub node: Account<'info, Node>,

    // On vérifie que le signataire est soit le propriétaire du client, soit du nœud
    #[account(
        constraint = user.key() == client.client_key || user.key() == node.node_key @ ErrorCode::Unauthorized
    )]
    pub user: Signer<'info>,
}



#[account]
#[derive(InitSpace)]
pub struct Node{
    pub node_key: Pubkey,

    pub in_session: bool,
    pub is_active: bool,
    pub solde: u64,
    pub price_hour: u64,
    #[max_len(44)]
    pub wg_key: String,
}

impl Node {
    pub fn create(&mut self, node_key: Pubkey, wg_key: String, price_hour: u64) {
        self.node_key = node_key;
        self.in_session = false;
        self.is_active = true;
        self.solde = 0;
        self.price_hour = price_hour;
        self.wg_key = wg_key;
        
    }
}

#[derive(InitSpace)]
#[account]
pub struct Client{
    pub client_key: Pubkey,

    pub nonce: u64,
    pub solde: u64,
    pub in_session: bool,
    #[max_len(44)]
    pub wg_key: String,
}

impl Client {
    pub fn create(&mut self, client_key :  Pubkey, wg_key : String){
        self.client_key = client_key;
        self.nonce = 0;
        self.solde = 0;
        self.in_session = false;
        self.wg_key = wg_key;
    }
}

#[derive(InitSpace)]
#[account]
pub struct Session{
    pub start_time: i64,
    pub end_time: i64,
    pub client_key: Pubkey,
    pub node_key: Pubkey,
    pub price_hour: u64,
    #[max_len(10)]
    pub session_id: String,
}

impl Session {

    pub fn start(&mut self,client_key : Pubkey, node_key : Pubkey,price_hour : u64, session_id : String)->Result<()>{
        self.start_time = Clock::get()?.unix_timestamp;
        self.end_time = 0;
        self.client_key = client_key;
        self.node_key = node_key;
        self.price_hour = price_hour;
        self.session_id = session_id;

        Ok(())
    }
    
}



#[error_code]
pub enum ErrorCode {
    #[msg("Le compte Client ne correspond pas à la session.")]
    InvalidClient,
    #[msg("Le compte Node ne correspond pas à la session.")]
    InvalidNode,
    #[msg("Cette session est déjà terminée.")]
    SessionAlreadyEnded,
    #[msg("Seul le client ou le nœud de la session peut l'arrêter.")]
    Unauthorized,
}

// #[account]
// pub struct SessionManager {
//     sessions: HashMap<String, Session>,
// }
// #[account]
// pub struct NodeManager {
//     nodes: HashMap<String, Node>,
// }