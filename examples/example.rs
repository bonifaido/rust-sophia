extern crate sophia;

use sophia::Native;

fn main() {
    println!("## Setup ##");

    let env = sophia::Sophia::new().unwrap();
    println!("env type {}", env.get_type().unwrap());

    let ctl = env.ctl();

    let res = ctl.set("sophia.path", "./target/test.db");
    println!("ctl.set {:?}", res.unwrap());
    let res = ctl.set("db", "test");
    println!("ctl.set {:?}", res.unwrap());

    let res = env.open();
    println!("env.open {:?}", res.unwrap());

    let db = ctl.get_db("db.test").unwrap();

    println!("## Set ##");
    {
        let object = db.object().unwrap();
        println!("object type {}", object.get_type().unwrap());

        let res = object.set("key", "hello".as_bytes());
        println!("object.set.key {:?}", res.unwrap());

        let res = object.set("value", "world".as_bytes());
        println!("object.set.value {:?}", res.unwrap());

        let res = db.set(&object);
        println!("db.set.object {:?}", res.unwrap());
    }

    println!("## Get ##");
    {
        let object = db.object().unwrap();

        let res = object.set("key", "hello".as_bytes());
        println!("object.set.key {:?}", res.unwrap());

        let object = db.get(&object).unwrap();
        println!("db.get.object Ok");

        let field = "value";
        let value = object.get(field).ok().expect("value can't be read");
        println!("object.get.{} {:?}", field, std::str::from_utf8(value).unwrap());
    }

    println!("## Transaction ##");
    {
        let object = db.object().unwrap();

        let res = object.set("key", "inside".as_bytes());
        println!("object.set.key {:?}", res.unwrap());

        let res = object.set("value", "transaction".as_bytes());
        println!("object.set.value {:?}", res.unwrap());

        let transaction = env.transaction().unwrap();

        let res = transaction.set(&object);
        println!("transaction.set.object {:?}", res.unwrap());

        let res = transaction.commit();
        println!("transaction.commit {:?}", res.unwrap());
    }

    println!("## Cursor ## Bogus atm -> thread '<main>' has overflowed its stack");
    {
        let options = db.object().unwrap();

        let cursor = db.cursor(&options).unwrap();
        println!("db.cursor Ok");

        loop {
            let object = cursor.get(); // TODO
            match object {
                Err(_) => break,
                Ok(object) => {
                    let key = object.get("key").unwrap();
                    let value = object.get("value").unwrap();
                    println!("cursor.object.key = {:?}", std::str::from_utf8(key).unwrap());
                    println!("cursor.object.value = {:?}", std::str::from_utf8(value).unwrap());
                }
            }
        }
    }

    println!("## Delete ##");
    {
        let object = db.object().unwrap();

        let res = object.set("key", "hello".as_bytes());
        println!("object.set.key {:?}", res.unwrap());

        let res = db.delete(&object).unwrap();
        println!("db.delete.object {:?}", res);
    }
}