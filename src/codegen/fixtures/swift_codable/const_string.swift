import Foundation

public enum ModelError: Error {
    case Error
}

// Const def for string
public enum Reason: String {
    // Everything is ok
    case Ok = "ok"
    // Request is bad
    case Error = "error"
}