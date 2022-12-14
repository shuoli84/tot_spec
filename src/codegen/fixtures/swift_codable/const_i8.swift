import Foundation

public enum ModelError: Error {
    case Error
}

// Const def for i8
// Second line of comment
public enum Code: Int8 {
    // Everything is ok
    case Ok = 0
    // Request is bad
    case Error = 1
}