use lalrpop_util::ParseError;
use crate::ast::Ident;
use crate::ast::Location;
use crate::token::Tok;
use crate::error::*;

pub fn make_config_comment(
    com_loc: Location,
    stmt_loc: Location,
    nac3com_above: Vec<(Ident, Tok)>,
    nac3com_end: Option<Ident>
) -> Result<Vec<Ident>, ParseError<Location, Tok, LexicalError>> {
    if com_loc.column() != stmt_loc.column() {
        return Err(ParseError::User {
            error: LexicalError {
                location: com_loc,
                error: LexicalErrorType::OtherError(
                    format!(
                        "config comment at top must have the same indentation with what it applies, comment at {}, statement at {}",
                        com_loc,
                        stmt_loc,
                    )
                )
            }
        })
    };
    Ok(
        nac3com_above
            .into_iter()
            .map(|(com, _)| com)
            .chain(nac3com_end.map_or_else(|| vec![].into_iter(), |com| vec![com].into_iter()))
            .collect()
    )
}