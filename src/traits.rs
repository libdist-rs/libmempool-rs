pub trait Sealer<Tx> 
{
    fn seal(&mut self) -> Vec<Tx>;
}