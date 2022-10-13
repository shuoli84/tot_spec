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

 // Base
protocol Base {
    var request_id: String? {
        get
        set
    }
}

 // Number
enum Number: Codable {
    case I64(Int64)
    case F64(Float64)

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
            
        }
        
    }
    
}

 // AddRequest
struct AddRequest: Codable, Base {
    var request_id: String?
    var numbers: [Number]?
}

 // ResetRequest
struct ResetRequest: Codable, Base {
    var request_id: String?
}

func foo() {
    let encoder = JSONEncoder()
    encoder.outputFormatting = .prettyPrinted
    
    let decoder = JSONDecoder()

    do {
        let data = SimpleStruct(bool_value: true, i8_value: 123, i64_value: 3333)
        let json_data = try! encoder.encode(data)
        print(String(data: json_data, encoding: .utf8)!)
    }

    do {
        let json_data = try! encoder.encode(Number.I64(23))
        print(String(data: json_data, encoding: .utf8)!)
        
        let data_back = try! decoder.decode(Number.self, from: json_data)
        print("data back \(data_back)")
    }
    
    do {
        let json_data = try! encoder.encode(Number.F64(23.123))
        print(String(data: json_data, encoding: .utf8)!)
        
        let data_back = try! decoder.decode(Number.self, from: json_data)
        print("data back \(data_back)")
    }

    do {
        let json_data = try! encoder.encode(ResetRequest(request_id: "hello"))
        print(String(data: json_data, encoding: .utf8)!)
        
        let data_back = try! decoder.decode(ResetRequest.self, from: json_data)
        print("data back \(data_back)")
    }
}

foo()
