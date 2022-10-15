import Foundation

enum ModelError: Error {
    case Error
}

 // SimpleStruct
struct SimpleStruct: Codable {
    var bool_value: Bool
    var i8_value: Int8
    var i64_value: Int64?
    var string_value: String?
    var bytes_value: Data?
    var string_to_string: [String:String]?
    var key_values: KeyValue?
    var children_container: Container?
    var children: [SimpleStruct]?
}

 // KeyValue
typealias KeyValue = [String:Data]

 // Container
typealias Container = [SimpleStruct]

 // RealNumber
struct RealNumber: Codable {
    var real: Float64?
    var imagine: Float64?
}

 // Number
enum Number: Codable {
    case I64(Int64)
    case F64(Float64)
    case RealNumber(RealNumber)

    // coding keys
    enum CodingKeys: String, CodingKey {
        case type, payload
    }
    
    // decoder
    init(from decoder: Decoder) throws {
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
                let payload = try container.decode(RealNumber.self, forKey:.payload)
                self = .RealNumber(payload)
            
            default:
                throw ModelError.Error
            
        }
    }
    
    
    // encoder
    func encode(to encoder: Encoder) throws {
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

 // BaseRequest
protocol BaseRequest {
    var request_id: String? {
        get
        set
    }
}

 // AddRequest
struct AddRequest: Codable, BaseRequest {
    var request_id: String?
    var numbers: [Number]?
}

 // ResetRequest
struct ResetRequest: Codable, BaseRequest {
    var request_id: String?
}
