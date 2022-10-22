import Foundation

public enum ModelError: Error {
    case Error
}

public struct SimpleStruct: Codable {
    public var bool_value: Bool
    public var i8_value: Int8
    public var i64_value: Int64?
    public var string_value: String?
    public var bytes_value: Data?
    public var string_to_string: [String:String]?
    public var key_values: SpecModel.KeyValue?
    public var children_container: SpecModel.Container?
    public var children: [SpecModel.SimpleStruct]?

    public init(bool_value: Bool, i8_value: Int8, i64_value: Int64? = nil, string_value: String? = nil, bytes_value: Data? = nil, string_to_string: [String:String]? = nil, key_values: SpecModel.KeyValue? = nil, children_container: SpecModel.Container? = nil, children: [SpecModel.SimpleStruct]? = nil) {
        self.bool_value = bool_value
        self.i8_value = i8_value
        self.i64_value = i64_value
        self.string_value = string_value
        self.bytes_value = bytes_value
        self.string_to_string = string_to_string
        self.key_values = key_values
        self.children_container = children_container
        self.children = children
    }
}

public typealias KeyValue = [String:Data]

public typealias Container = [SpecModel.SimpleStruct]

public struct RealNumber: Codable {
    public var real: Float64?
    public var imagine: Float64?

    public init(real: Float64? = nil, imagine: Float64? = nil) {
        self.real = real
        self.imagine = imagine
    }
}

public enum Number: Codable {
    case I64(Int64)
    case F64(Float64)
    case RealNumber(SpecModel.RealNumber)

    // coding keys
    enum CodingKeys: String, CodingKey {
        case type, payload
    }

    // decoder
    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(String.self, forKey: CodingKeys.type)
        switch type {
            case "I64":
                let payload = try container.decode(Int64.self, forKey:.payload)
                self = .I64(payload)
            case "F64":
                let payload = try container.decode(Float64.self, forKey:.payload)
                self = .F64(payload)
            case "RealNumber":
                let payload = try container.decode(SpecModel.RealNumber.self, forKey:.payload)
                self = .RealNumber(payload)
            default:
                throw ModelError.Error
        }
    }

    // encoder
    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
            case let .I64(payload):
                try container.encode("I64", forKey: .type)
                try container.encode(payload, forKey: .payload)
            case let .F64(payload):
                try container.encode("F64", forKey: .type)
                try container.encode(payload, forKey: .payload)
            case let .RealNumber(payload):
                try container.encode("RealNumber", forKey: .type)
                try container.encode(payload, forKey: .payload)
        }
    }
}

public protocol BaseRequest {
    var request_id: String? {
        get
        set
    }
}

public struct AddRequest: Codable, BaseRequest {
    public var request_id: String?
    public var numbers: [SpecModel.Number]?

    public init(request_id: String? = nil, numbers: [SpecModel.Number]? = nil) {
        self.request_id = request_id
        self.numbers = numbers
    }
}

public struct ResetRequest: Codable, BaseRequest {
    public var request_id: String?

    public init(request_id: String? = nil) {
        self.request_id = request_id
    }
}

public enum ConstInteger: Int64 {
    case Value1 = 1
    case Value2 = 2
}
