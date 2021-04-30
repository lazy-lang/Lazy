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
    Types.Type1 => {
        print "My type is a string which contains: " + myType;
    },
    Types.Type2 => {
        print "My type is a number which contains: " + myType;
    },
    Types.Type3 => {
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

### tuples

```
struct Something {
    things: [str, int]
}

let me = Something{things: ["Google", 19] };
me.things.0;
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

    AgeUp => {
        state.age++;
    }

    ChangeName(name: str) => {
        state.name = name;
    }

    Print => {
        print "Name: ${state.name} Age: ${state.age}\n";
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
    next: Option<Person>
}

actor PersonList with Option<Node> {

    Add(person: Person) => {
        match state {
            None => state = Node<T>{value: person},
            Some => state.next = Node<T>{value: person}
        }
    }

    AgeUp => {
        let node = state;
        while node {
            emit node->AgeUp;
            node = node.next;
        }
    }

    Print => {
        let node = state;
        while node {
            emit node->Print;
        }
    }

}

let people = PersonList;

emit people->Add {person: Person{name: "Google", age: 19} };
emit people->Add {person: Person{name: "You", age: 21} };

emit people->AgeUp;

emit people->Print;
```

## Partial types (SomeOf)

The `SomeOf` type lets you pass structs and actors which are different, but have common methods / fields.

```
struct Animal {
    age: int,
    name: str
}

struct Human {
    age: int,
    name: str
}

let me: SomeOf<{age: int}> = Animal{age: 3, name: "Google"}; // You will only be able to access the `age` field.

me.name; // Error!
me.age; // 3

me = Human{age: 19, name: "Google"};
me.age; // 19
```

Same can be done with actors.

```
actor Server with SomeOf<{req: (path: str)}> {

    Server(handler: SomeOf<{req: (path: str)}>) {
        state = handler;
    }

    // Some internal events which emit state->req
}

actor ServerHandler {

    req(path: str) {
        print path;
    }

}

Server{handler: ServerHandler};
```

