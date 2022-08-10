use concordium_std::*;

/**
 ** Need to keep track of whose turn it is, and if a game is active or ended.
 **/

#[derive(Serialize, PartialEq, Eq, Debug, Clone, Copy)]
enum Mark {
    X,
    O,
}

#[derive(Serialize)]
enum Status {
    Win(Mark),
    Draw,
    Running,
}

type Board = [Option<Mark>; 9];

#[derive(Serialize)]
struct State {
    player_x: Option<AccountAddress>,
    player_o: Option<AccountAddress>,
    b: Board,
    buy_in: Amount,
    active: bool,
    turn: Mark,
}

#[init(contract = "TicTacToe", parameter = "Amount")]
fn ttt_init<S: HasStateApi>(
    ctx: &impl HasInitContext,
    _state_builder: &mut StateBuilder<S>,
) -> InitResult<State> {
    let bi: Amount = ctx.parameter_cursor().get()?;
    Ok(State {
	player_x: None,
	player_o: None,
	b: [None; 9],
	buy_in: bi,
	active: false,
	turn: Mark::X, // Defaulting to X starting
    })
}

#[receive(contract = "TicTacToe", name = "join", payable, mutable)]
fn ttt_join<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<State, StateApiType = S>,
    amount: Amount
) -> ReceiveResult<()> {
    let b = host.self_balance();
    let mut state = host.state_mut();
    ensure!(!state.active);
    ensure!(state.buy_in == amount);
    
    match ctx.sender() {
	Address::Contract(_) => bail!(),
	Address::Account(player) => {
	    match state.player_x {
		None => {
		    ensure!(b == state.buy_in);
		    state.player_x = Some(player)
		},
		Some(_) => {
		    state.player_o = Some(player);
		    ensure!(b == (2*state.buy_in));
		    state.active = true
		}
	    }
	}
    }
    Ok(())
}

#[receive(contract = "TicTacToe", name = "place", parameter = "u8", mutable)]
fn ttt_place<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host : &mut impl HasHost<State, StateApiType = S>,
) -> ReceiveResult<(Board,Status)> {
    let b = host.self_balance();
    ensure!(host.state().active); // Game should be running
    let px = host.state().player_x.expect("active game should have player X.");
    let po = host.state().player_x.expect("active game should have player O.");
    
    match host.state().turn {
	Mark::X => ensure!(ctx.sender().matches_account(&px)),
	Mark::O => ensure!(ctx.sender().matches_account(&po)),
    };

    let mv: u8 = ctx.parameter_cursor().get()?;
    let pos = usize::from(mv);
    
    ensure!(host.state().b[pos].is_none());
    host.state_mut().b[pos] = Some(host.state().turn);

    match host.state().turn {
	Mark::X => host.state_mut().turn = Mark::O,
	Mark::O => host.state_mut().turn = Mark::X,
    };

    let status = check_winner(&host.state().b);
    
    match status {
	Status::Win(Mark::X) => {
	    host.invoke_transfer(&px, b)?;
	    host.state_mut().active = false;
	    host.state_mut().turn = Mark::X
	},
	Status::Win(Mark::O) => {
	    host.invoke_transfer(&po, host.self_balance())?;
	    host.state_mut().active = false;
	    host.state_mut().turn = Mark::X
	},
	Status::Draw => {
	    let pay = Amount::from_ccd(host.self_balance().micro_ccd / 2);
	    host.invoke_transfer(&px, pay)?;
	    host.invoke_transfer(&po, pay)?;
	    host.state_mut().active = false;
	    host.state_mut().turn = Mark::X
	},
	Status::Running => ()
    }

    Ok((host.state().b,status))
}

fn check_winner (b: &Board) -> Status {
    // There MUST be a better way to check this
    if b[0].is_some() {
	if b[0] == b[1] && b[0] == b[2] {
	    return Status::Win(b[0].unwrap())
	} else if b[0] == b[3] && b[0] == b[6] {
	    return Status::Win(b[0].unwrap())
	}
    }
    if b[4].is_some() {
	if b[0] == b[4] && b[4] == b[8] {
	    return Status::Win(b[4].unwrap())
	} else if b[1] == b[4] && b[4] == b[7] {
	    return Status::Win(b[4].unwrap())
	} else if b[2] == b[4] && b[4] == b[6] {
	    return Status::Win(b[4].unwrap())
	} else if b[3] == b[4] && b[4] == b[5] {
	    return Status::Win(b[4].unwrap())
	}
    }
    if b[8].is_some() {
	if b[2] == b[5] && b[5] == b[8] {
	    return Status::Win(b[8].unwrap())
	} else if b[6] == b[7] && b[7] == b[8] {
	    return Status::Win(b[8].unwrap())
	}
    }
    if b.into_iter().all(|&x| x.is_some()) {
	return Status::Draw
    }
    return Status::Running
}   


/*
#[cfg(test)]
mod tests {
    use super::*;
    use test_infrastructure::*;

    #[test]
    fn test_init() {
	let ctx = TestInitContext::empty();
	let mut state_builder = TestStateBuilder::new();
	let state_result = ttt_init(&ctx, &mut state_builder);
	let state = state_result.expect("Contract initialization results in error.");
	assert_eq!(
	    state,
	    [None; 9],
	    "Board should be empty after initialization."
	);
    }

    #[test]
    fn test_place() {
	let mut ctx = TestReceiveContext::empty();
	let mv = Move {
	    mark: Mark::X,
	    pos: 4
	};

	let parameter_bytes = to_bytes(&mv);
	ctx.set_parameter(&parameter_bytes);

	let mut host: TestHost<Board> = TestHost::new(
	    [None;9],
	    TestStateBuilder::new(),
	);

	let result = ttt_place(&ctx, &mut host);
	let b = result.expect("Placing a mark results in an error.");

	let a = [None,None,None,
		 None,Some(Mark::X),None,
		 None,None,None];
	
	assert_eq!(b,a,"The board should have an X in the middle and no other marks.");
    }
}
*/
