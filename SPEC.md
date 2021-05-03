# Lazy

Lazy is a staticly typed, **actor orianted** programming language. In lazy, there are two types of values: `data` and `actors`.

## Data

`data` includes primitive values, structs and enums. 

### bool

```
let isCool = true;
```

### numbers

```
let age = 18; // Inferred type: int
let pi: double  = 3.14; // Static type: double
let minusPi = -pi; // Inferred type: double
```

All number types: `num`, `float`, `double`, `int`.

### strings

Type: `str`.

```
let hello = "World!";
```

### enums

```
enum Types {
    Type1: str,
    Type2: num,
    Type3
}

let myType = Types.Type1{"Hi!"};
myType = Types.Type2{3.14};
myType = Types.Type3;

match myType {
    Types.Type1: {
        print "My type is a string which contains: " + myType;
    },
    Types.Type2: {
        print "My type is a number which contains: " + myType;
    },
    Types.Type3: {
        print "My type is empty...!"
    }
}
```

### structs

```
struct Person {
    name: str,
    age: int
}

let me = Person{name: "Google", age: 19};

me.name;
me.age;
```

## Actors

An `actor` is very similar to a **process** in erlang. Every `actor` has it's own memory region which is isolated, and a single state, which only the actor can access, which acts as the single source of truth.

```
struct PersonObj {
    name: str,
    age: int
}


//The state of this actor is a "_Person" struct, and "instruction" is the argument for when the actor gets called 
actor Person with PersonObj {

    Person(name: str, age: int) => { // Constructor, only arg must be initial state, required if actor has state
        state = PersonObj{name, age};
    }

    on AgeUp => {
        state.age++;
    }

    on ChangeName(name: str) => {
        state.name = name;
    }

    on Print => {
        emit Console->println{ msg: "Name: ${state.name} Age: ${state.age}\n" };
    }

}

let me = Person{name: "Google", age: 19};

emit me->AgeUp;
emit me->ChangeName{name: "GoogleFeud"}; 
emit me->Print;
```

### emit

The `emit` keyword emits an event to the actor. 

`emit me->ChangeName{name: "GoogleFeud"};`

`me->ChangeName` is the path to the event. 
`{name: "GoogleFeud"}` are the event arguments.


## Arrays? Hashmaps?

Arrays and hashmaps just don't make **sense** with the actor model. If you want a place to store a bunch of instances of a model or a struct, make a specific actor.

```

struct Node {
    value: Person,
    next: Node?
}

actor PersonList with Node? {

    on Add(person: Person) => {
        match state {
            None: state = Node{value: person},
            Some: {
                let new_node = Node{value: person}
                new_node.next = state;
                state = new_node;
            }
        }
    }

    on AgeUp => {
        let node = state;
        while node {
            emit node->AgeUp;
            node = node.next;
        }
    }

    on Print => {
        let node = state;
        while node {
            emit node->Print;
        }
    }

}

let people = PersonList{};

emit people->Add { person: Person{name: "Google", age: 19} };
emit people->Add { person: Person{name: "You", age: 21} };

emit people->AgeUp;

emit people->Print;
```

## Partial types

You can also pass structs and actors which are different, but have common methods / fields.

```
struct Animal {
    age: int,
    name: str
}

struct Human {
    age: int,
    name: str
}

let me: {age: int} = Animal{age: 3, name: "Google"}; // You will only be able to access the `age` field.

me.name; // Error!
me.age; // 3

me = Human{age: 19, name: "Google"};
me.age; // 19
```

Same can be done with actors.

```
actor Server with {req: (path: str)} {

    Server(handler: {req: (path: str)}) {
        state = handler;
    }

    // Some internal events which emit state->req
}

actor ServerHandler {

    on req(path: str) {
        print path;
    }

}

Server{handler: ServerHandler{} };
```

## Singleton actors

You can use the `single` keyword to make it so an `actor` is a singleton.

```
struct _Db {
    requests: RequestPool
}

single actor Db with _Db {

    Db() {
        state = ...
    }

    on insert(data: { id: int }) {
        emit state->make { table: "USERS", data };
    }

}

// No need to instantiate the actor
emit Db->insert { id: 123 };
```