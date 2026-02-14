# tot-spec Language Examples

Complete examples of generated code for each supported language.

## Rust (rs_serde)

### Generated Code

```rust
/// User
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: std::string::String,
    pub age: std::option::Option<i32>,
}

/// CreateUserRequest
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CreateUserRequest {
    pub username: std::string::String,
    pub email: std::option::Option<std::string::String>,
}

/// CreateUserResponse
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CreateUserResponse {
    pub user_id: std::string::String,
}

/// GetUserRequest
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GetUserRequest {
    pub user_id: std::string::String,
}

/// GetUserResponse
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GetUserResponse {
    pub user: User,
}
```

### Usage Example

```rust
use serde_json;

let user = User {
    id: "123".to_string(),
    age: Some(30),
};

let json = serde_json::to_string(&user)?;
let deserialized: User = serde_json::from_str(&json)?;

let request = CreateUserRequest {
    username: "alice".to_string(),
    email: Some("alice@example.com".to_string()),
};
```

## Python (py_dataclass)

### Generated Code

```python
from __future__ import annotations
from dataclasses import dataclass, field
import typing

@dataclass
class User:
    id: str
    age: typing.Optional[int] = None

    def to_dict(self):
        return {"id": self.id, "age": self.age}

    @staticmethod
    def from_dict(d):
        return User(id=d["id"], age=d.get("age"))

@dataclass
class CreateUserRequest:
    username: str
    email: typing.Optional[str] = None

    def to_dict(self):
        return {"username": self.username, "email": self.email}

    @staticmethod
    def from_dict(d):
        return CreateUserRequest(username=d["username"], email=d.get("email"))

@dataclass
class CreateUserResponse:
    user_id: str

    def to_dict(self):
        return {"user_id": self.user_id}

    @staticmethod
    def from_dict(d):
        return CreateUserResponse(user_id=d["user_id"])

@dataclass
class GetUserRequest:
    user_id: str

    def to_dict(self):
        return {"user_id": self.user_id}

    @staticmethod
    def from_dict(d):
        return GetUserRequest(user_id=d["user_id"])

@dataclass
class GetUserResponse:
    user: User

    def to_dict(self):
        return {"user": self.user.to_dict() if self.user else None}

    @staticmethod
    def from_dict(d):
        return GetUserResponse(user=User.from_dict(d["user"]) if d.get("user") else None)
```

### Usage Example

```python
import json

user = User(id="123", age=30)
json_str = json.dumps(user.to_dict())
deserialized = User.from_dict(json.loads(json_str))

request = CreateUserRequest(username="alice", email="alice@example.com")
```

## TypeScript

### Generated Code

```typescript
export interface User {
    id: string;
    age?: number;
}

export interface CreateUserRequest {
    username: string;
    email?: string;
}

export interface CreateUserResponse {
    user_id: string;
}

export interface GetUserRequest {
    user_id: string;
}

export interface GetUserResponse {
    user: User;
}
```

### Usage Example

```typescript
const user: User = { id: "123", age: 30 };
const json = JSON.stringify(user);
const deserialized = JSON.parse(json) as User;

const request: CreateUserRequest = {
    username: "alice",
    email: "alice@example.com"
};
```

## Swift (swift_codable)

### Generated Code

```swift
import Foundation

public struct User: Codable {
    public var id: String
    public var age: Int?

    public init(id: String, age: Int? = nil) {
        self.id = id
        self.age = age
    }
}

public struct CreateUserRequest: Codable {
    public var username: String
    public var email: String?

    public init(username: String, email: String? = nil) {
        self.username = username
        self.email = email
    }
}

public struct CreateUserResponse: Codable {
    public var user_id: String

    public init(user_id: String) {
        self.user_id = user_id
    }
}

public struct GetUserRequest: Codable {
    public var user_id: String

    public init(user_id: String) {
        self.user_id = user_id
    }
}

public struct GetUserResponse: Codable {
    public var user: User

    public init(user: User) {
        self.user = user
    }
}
```

### Usage Example

```swift
import Foundation

let user = User(id: "123", age: 30)
let encoder = JSONEncoder()
let data = try encoder.encode(user)
let json = String(data: data, encoding: .utf8)

let request = CreateUserRequest(username: "alice", email: "alice@example.com")
```

## Java (java_jackson)

### Generated Code

```java
package com.example;

public class User {
    private String id;
    private Integer age;

    public String getId() { return id; }
    public void setId(String id) { this.id = id; }

    public Integer getAge() { return age; }
    public void setAge(Integer age) { this.age = age; }
}

public class CreateUserRequest {
    private String username;
    private String email;

    public String getUsername() { return username; }
    public void setUsername(String username) { this.username = username; }

    public String getEmail() { return email; }
    public void setEmail(String email) { this.email = email; }
}

public class CreateUserResponse {
    private String userId;

    public String getUserId() { return userId; }
    public void setUserId(String userId) { this.userId = userId; }
}

public class GetUserRequest {
    private String userId;

    public String getUserId() { return userId; }
    public void setUserId(String userId) { this.userId = userId; }
}

public class GetUserResponse {
    private User user;

    public User getUser() { return user; }
    public void setUser(User user) { this.user = user; }
}
```

### Usage Example

```java
import com.fasterxml.jackson.databind.ObjectMapper;

ObjectMapper mapper = new ObjectMapper();

User user = new User();
user.setId("123");
user.setAge(30);
String json = mapper.writeValueAsString(user);

CreateUserRequest request = new CreateUserRequest();
request.setUsername("alice");
request.setEmail("alice@example.com");
```

## Enum Example

### Input YAML

```yaml
models:
  - name: PaymentMethod
    type:
      name: enum
      variants:
        - name: CreditCard
          payload_type: string
        - name: PayPal
        - name: BankTransfer
          payload_type: string
```

### Rust Generated

```rust
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum PaymentMethod {
    CreditCard(String),
    PayPal,
    BankTransfer(String),
}
```

### Python Generated

```python
@dataclass
class PaymentMethod:
    type: str
    payload: typing.Optional[typing.Any] = None

    @staticmethod
    def credit_card(value: str):
        return PaymentMethod(type="CreditCard", payload=value)

    @staticmethod
    def pay_pal():
        return PaymentMethod(type="PayPal", payload=None)

    @staticmethod
    def bank_transfer(value: str):
        return PaymentMethod(type="BankTransfer", payload=value)
```

### TypeScript Generated

```typescript
export type PaymentMethod =
    | { type: "CreditCard", payload: string }
    | { type: "PayPal", payload?: never }
    | { type: "BankTransfer", payload: string };
```
