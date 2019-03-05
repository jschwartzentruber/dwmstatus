use crate::prelude::*;
use libc::{getloadavg, c_double};

pub fn status() -> String {
    let mut result = "".to_string();
    let mut avgs: Vec<c_double> = vec![0.0; 3];
    perror_check!(getloadavg(avgs.as_mut_ptr(), 3));
    if avgs[0] >= 5.0 {
        result += BAD;
    }
    result += &format!("{:0.2} {:0.2} {:0.2}", avgs[0], avgs[1], avgs[2]);
    result
}
