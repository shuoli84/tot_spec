import Foundation

public enum ModelError: Error {
    case Error
}

public typealias BaseId = Int64

public struct PageInfo: Codable {
    public var page: Int32
    public var page_size: Int32

    public init(page: Int32, page_size: Int32) {
        self.page = page
        self.page_size = page_size
    }
}
