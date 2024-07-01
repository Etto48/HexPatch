use super::header_parser_info::HeaderParserInfo;

#[derive(Debug, Clone, Default)]
pub struct ExportedHeaderParsers
{
    pub headers: Vec<HeaderParserInfo>,
}

impl ExportedHeaderParsers
{
    pub fn add_header_parser(&mut self, parser: String)
    {
        self.headers.push(HeaderParserInfo {parser});
    }

    pub fn remove_header_parser(&mut self, parser: &str) -> bool
    {
        if let Some(index) = self.headers.iter().position(|c| c.parser == parser)
        {
            self.headers.remove(index);
            true
        }
        else
        {
            false
        }
    }

    pub fn take(&mut self) -> Self
    {
        std::mem::take(self)
    }
}