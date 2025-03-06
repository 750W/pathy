use crate::bezier::BezPoint;

/// Generates path code from a path.
pub fn generate(path: &[BezPoint], step: f32) -> String {
    if path.len() < 2 {
        return format!("std::vector<wolflib::Moment> = wolf.solve({{}}, {step});");
    }
    let mut result: String = "std::vector<wolflib::Moment> = wolf.solve({\n".into();
    for idx in 0..path.len() - 1 {
        let p1 = &path[idx];
        let p2 = &path[idx + 1];
        result.push_str(
            format!(
                "    {{{{{:.3}_in, {:.3}_in}}, {{{:.3}_in, {:.3}_in}}, {{{:.3}_in, {:.3}_in}}, {{{:.3}_in, {:.3}_in}}}}",
                p1.pos.borrow().x,
                p1.pos.borrow().y,
                p1.cp2.borrow().x,
                p1.cp2.borrow().y,
                p2.cp1.borrow().x,
                p2.cp1.borrow().y,
                p2.pos.borrow().x,
                p2.pos.borrow().y
            )
            .as_str(),
        );
        if idx < path.len() - 2 {
            result.push_str(",\n");
        }
    }
    result.push_str(format!("}}, {step});").as_str());
    result
}
