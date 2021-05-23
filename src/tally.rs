use std::collections::BTreeMap;
use crate::db::Db;
use crate::db::entities::Candidate;

pub async fn tally(db: &Db) -> sqlx::Result<(String, Vec<(String, String, bool)>)>{
    let mut tally = Tally::new(db).await?; //TODO: use a transaction to prevent the data from changing while we read it
    let mut round_id = 1;

    let description;
    let mut fields = Vec::new();

    loop{
        let field_title = format!("The results from round {} are:", round_id);

        let provisional_tally = tally.tally_current_round();

        let mut sorted_tally = provisional_tally.iter().collect::<Vec<(&(String, u32), &u32)>>();
        sorted_tally.sort_by(|a, b|{
            a.0.1.partial_cmp(&(b.0.1)).unwrap()
        });

        let mut field_description = String::new();
        for ((name, _id), count) in sorted_tally{
            field_description.push_str(&format!(" - {} with {} votes\n", name, count));
        }

        let majority = Tally::majority(&provisional_tally);
        if let Some(majority) = majority{
            description = format!("\n**A winner has been selected after {} rounds: __{}__\n**", round_id, majority);
            fields.push((field_title, field_description, false));
            break;
        }

        let least_popular = tally.least_popular(&provisional_tally);
        match least_popular{
            Some(least_popular) => {
                let name = &tally.options.get(&least_popular).unwrap().name;
                field_description.push_str(&format!("\nRedistributing votes from the least popular option: {}\n\n", name));
                tally.eliminate(least_popular).await?;

                fields.push((field_title, field_description, false));
            },
            None => {
                description = "It looks like no votes are left, aborting.\n".into();
                fields.push((field_title, field_description, false));
                break;
            }
        }

        round_id += 1;
    }

    Ok((description, fields))
}

struct VoterData{
    choice_number: u32,
    option: Option<u32>
}

struct Tally<'a>{
    db: &'a Db,
    options: BTreeMap<u32, Candidate>,
    data: BTreeMap<u64, VoterData>
}

impl<'a> Tally<'a>{

    async fn new(db: &'a Db) -> sqlx::Result<Tally<'a>>{
        let mut options = BTreeMap::new();
        for candidate in db.list_candidates().await?{
            options.insert(candidate.id, candidate);
        }

        let mut data = BTreeMap::new();
        for vote in db.get_1st_votes().await?.iter(){
            data.insert(vote.user, VoterData{
                choice_number: 1,
                option: Some(vote.option)
            });
        }

        Ok(
            Self{
                db,
                options,
                data
            }
        )
    }

    fn tally_current_round(&self) -> BTreeMap<(String, u32), u32>{
        //count
        let mut running_totals = BTreeMap::new();

        for key in self.options.keys(){
            running_totals.insert(*key, 0);
        }

        for value in self.data.values(){
            if let Some(option) = &value.option{
                *running_totals.get_mut(option).unwrap() += 1;
            }
        }

        //convert to strings
        let mut strings = BTreeMap::new();

        for (id, candidate) in &self.options{
            strings.insert((candidate.name.clone(), *id), *running_totals.get(id).unwrap());
        }

        strings
    }

    fn majority(tally: &BTreeMap<(String, u32), u32>) -> Option<String>{
        //calculate total votes
        let mut total_votes = 0;
        for (_, count) in tally{
            total_votes += count;
        }

        //determine number of needed votes
        let needed = ((total_votes as f64)/2.0).ceil() as u32;

        //determine if the threshold is reached
        let mut majority = None;
        for ((name, _id), count) in tally{
            if *count >= needed{
                if majority.is_some(){
                    return None;
                }

                majority = Some(name.into());
            }
        }

        majority
    }

    fn least_popular(&self, tally: &BTreeMap<(String, u32), u32>) -> Option<u32>{
        let mut least_popular_count = 0;
        let mut least_popular = None;

        for ((_name, id), count) in tally{
            if *count != 0 && (least_popular.is_none() || *count < least_popular_count){
                least_popular = Some(*id);
                least_popular_count = *count;
            }
        }

        least_popular
    }

    async fn eliminate(&mut self, id: u32) -> sqlx::Result<()>{
        for (user_id, data) in &mut self.data{
            if let Some(option) = data.option{
                if option == id {
                    data.choice_number += 1;
                    data.option = self.db.get_nth_vote(*user_id, data.choice_number).await?;
                }
            }
        }
        Ok(())
    }

}