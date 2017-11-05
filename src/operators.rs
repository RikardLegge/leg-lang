use parser::{AstOperatorCall, AstOperator, AstNodeType, AstNumberValue};
use interp::{InterpValue, InterpError};

pub fn apply_operation(lhs: InterpValue, rhs: InterpValue, operator: AstOperator) -> Result<InterpValue, InterpError> {
    use interp::InterpValue::*;

    let res = match (lhs, rhs) {
        (InterpNumber(lhs), InterpNumber(rhs)) => {
            apply_number_number_operation(lhs, rhs, operator)
        }
        (tp1, tp2) => {
            let msg = format!("Operator not yet implemented. lhs: {:?}, rhs: {:?}", tp1, tp2);
            return Err(InterpError::new(msg));
        }
    };
    return Ok(res);
}

fn apply_number_number_operation(lhs: f64, rhs: f64, operator: AstOperator) -> InterpValue {
    use parser::AstOperator::*;
    let val = match operator {
        Add => { lhs + rhs }
        Sub => { lhs - rhs }
        Mult => { lhs * rhs }
        Div => { lhs / rhs }
        Pow => { lhs.powf(rhs) }
        Mod => { lhs % rhs }
    };
    return InterpValue::InterpNumber(val);
}
