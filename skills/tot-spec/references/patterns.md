# Common Patterns for tot-spec

Reusable patterns and conventions for defining models and RPC services.

## Pagination

### Standard Pagination Request

```yaml
- name: ListUsersRequest
  type: struct
  fields:
    - name: page
      type: i32
    - name: page_size
      type: i32
    - name: order_by
      type: string
    - name: filter
      type: json
```

### Standard Pagination Response

```yaml
- name: ListUsersResponse
  type: struct
  fields:
    - name: items
      type: list[User]
      required: true
    - name: total
      type: i64
      required: true
    - name: page
      type: i32
      required: true
    - name: page_size
      type: i32
      required: true

methods:
  - name: ListUsers
    desc: "List users with pagination"
    request: ListUsersRequest
    response: ListUsersResponse
```

## Error Handling

### Error Response Type

```yaml
- name: ErrorResponse
  type: struct
  fields:
    - name: code
      type: i32
      required: true
    - name: message
      type: string
      required: true
    - name: details
      type: json

methods:
  - name: GetUser
    request: GetUserRequest
    response: GetUserResponse | ErrorResponse
```

### Status Code Constants

```yaml
- name: StatusCode
  type:
    name: const
    value_type: i32
    values:
      - name: Ok
        value: 0
      - name: InvalidArgument
        value: 3
      - name: NotFound
        value: 5
      - name: InternalError
        value: 13
```

## CRUD Operations

### Complete User Service

```yaml
models:
  - name: User
    type: struct
    fields:
      - name: id
        type: string
      - name: name
        type: string
      - name: email
        type: string
      - name: created_at
        type: i64

  - name: CreateUserRequest
    type: struct
    fields:
      - name: name
        type: string
        required: true
      - name: email
        type: string
        required: true

  - name: CreateUserResponse
    type: struct
    fields:
      - name: user
        type: User
        required: true

  - name: GetUserRequest
    type: struct
    fields:
      - name: user_id
        type: string
        required: true

  - name: GetUserResponse
    type: struct
    fields:
      - name: user
        type: User
        required: true

  - name: UpdateUserRequest
    type: struct
    fields:
      - name: user_id
        type: string
        required: true
      - name: name
        type: string
      - name: email
        type: string

  - name: UpdateUserResponse
    type: struct
    fields:
      - name: user
        type: User
        required: true

  - name: DeleteUserRequest
    type: struct
    fields:
      - name: user_id
        type: string
        required: true

  - name: DeleteUserResponse
    type: struct
    fields:
      - name: deleted
        type: bool
        required: true

methods:
  - name: CreateUser
    desc: "Create a new user"
    request: CreateUserRequest
    response: CreateUserResponse

  - name: GetUser
    desc: "Get user by ID"
    request: GetUserRequest
    response: GetUserResponse

  - name: UpdateUser
    desc: "Update user information"
    request: UpdateUserRequest
    response: UpdateUserResponse

  - name: DeleteUser
    desc: "Delete user by ID"
    request: DeleteUserRequest
    response: DeleteUserResponse
```

## Base Request Pattern (Virtual Types)

### Define Base Fields

```yaml
- name: BaseRequest
  type:
    name: virtual
    fields:
      - name: request_id
        type: string
      - name: timestamp
        type: i64
```

### Extend in Requests

```yaml
- name: CreateUserRequest
  type:
    name: struct
    extend: BaseRequest
    fields:
      - name: username
        type: string

- name: DeleteUserRequest
  type:
    name: struct
    extend: BaseRequest
    fields:
      - name: user_id
        type: string
```

## Domain-Specific Types (New Type)

### Type-Safe IDs

```yaml
- name: UserId
  type:
    name: new_type
    inner_type: string
  attributes:
    rs_extra_derive: Hash, PartialEq

- name: Email
  type:
    name: new_type
    inner_type: string

- name: CreateUserRequest
  type: struct
  fields:
    - name: user_id
      type: UserId
      required: true
    - name: email
      type: Email
      required: true
```

### Monetary Values

```yaml
- name: Amount
  type:
    name: new_type
    inner_type: decimal

- name: Currency
  type:
    name: const
    value_type: string
    values:
      - name: Usd
        value: "USD"
      - name: Eur
        value: "EUR"
      - name: Gbp
        value: "GBP"

- name: Money
  type: struct
  fields:
    - name: amount
      type: Amount
      required: true
    - name: currency
      type: Currency
      required: true
```

## Event Messages

### Event Envelope

```yaml
- name: Event
  type: struct
  fields:
    - name: id
      type: string
      required: true
    - name: type
      type: string
      required: true
    - name: timestamp
      type: i64
      required: true
    - name: data
      type: json
      required: true

- name: UserCreatedEvent
  type: struct
  fields:
    - name: user_id
      type: string
      required: true
    - name: username
      type: string
      required: true

methods:
  - name: PublishUserCreated
    desc: "Publish user created event"
    request: UserCreatedEvent
    response: Event
```

## Bulk Operations

### Bulk Request/Response

```yaml
- name: BulkCreateUsersRequest
  type: struct
  fields:
    - name: users
      type: list[CreateUserRequest]
      required: true

- name: BulkCreateUsersResponse
  type: struct
  fields:
    - name: users
      type: list[User]
      required: true
    - name: failed
      type: i32
      required: true

methods:
  - name: BulkCreateUsers
    desc: "Create multiple users"
    request: BulkCreateUsersRequest
    response: BulkCreateUsersResponse
```

## Authentication

### Auth Request Pattern

```yaml
- name: AuthRequest
  type:
    name: virtual
    fields:
      - name: api_key
        type: string
      - name: auth_token
        type: string

- name: GetUserRequest
  type:
    name: struct
    extend: AuthRequest
    fields:
      - name: user_id
        type: string
```

### Token Types

```yaml
- name: AccessToken
  type:
    name: new_type
    inner_type: string

- name: RefreshToken
  type:
    name: new_type
    inner_type: string

- name: AuthTokens
  type: struct
  fields:
    - name: access_token
      type: AccessToken
      required: true
    - name: refresh_token
      type: RefreshToken
      required: true
    - name: expires_in
      type: i64
      required: true
```

## Filtering and Sorting

### Generic Filter

```yaml
- name: Filter
  type: struct
  fields:
    - name: field
      type: string
      required: true
    - name: operator
      type: string
      required: true
    - name: value
      type: string

- name: SortOrder
  type:
    name: const
    value_type: string
    values:
      - name: Asc
        value: "asc"
      - name: Desc
        value: "desc"

- name: Sort
  type: struct
  fields:
    - name: field
      type: string
      required: true
    - name: order
      type: SortOrder
      required: true
```
