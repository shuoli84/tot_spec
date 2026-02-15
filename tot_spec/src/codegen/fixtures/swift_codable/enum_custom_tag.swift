import Foundation

public enum ModelError: Error {
    case Error
}

public enum Number: Codable {
    case Int64(Int64)
    case Float(Float64)
    case RealNumber(PACKAGE.RealNumber)

    // coding keys
    enum CodingKeys: String, CodingKey {
        case kind, data
    }

    // decoder
    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let kind = try container.decode(String.self, forKey: .kind)
        switch kind {
            case "Int64":
                let data = try container.decode(Int64.self, forKey:.data)
                self = .Int64(data)
            case "Float":
                let data = try container.decode(Float64.self, forKey:.data)
                self = .Float(data)
            case "RealNumber":
                let data = try container.decode(PACKAGE.RealNumber.self, forKey:.data)
                self = .RealNumber(data)
            default:
                throw ModelError.Error
        }
    }

    // encoder
    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
            case let .Int64(data):
                try container.encode("Int64", forKey: .kind)
                try container.encode(data, forKey: .data)
            case let .Float(data):
                try container.encode("Float", forKey: .kind)
                try container.encode(data, forKey: .data)
            case let .RealNumber(data):
                try container.encode("RealNumber", forKey: .kind)
                try container.encode(data, forKey: .data)
        }
    }
}

public struct RealNumber: Codable {
    public var part_0: Float64?
    public var part_1: Float64?

    public init(part_0: Float64? = nil, part_1: Float64? = nil) {
        self.part_0 = part_0
        self.part_1 = part_1
    }
}
