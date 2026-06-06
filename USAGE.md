# 签名验签服务使用说明

## 服务说明

本服务**仅提供签名和验签功能**，在本地执行，**不会请求星驿支付官方API**。

您的业务流程应该是：
1. **签名**：调用本服务的 `/sign` 接口获取签名，然后**自行请求星驿支付API**
2. **验签**：收到星驿支付的异步通知后，调用本服务的 `/verify` 接口验证签名

---

## 一、签名接口

### 接口地址
```
POST http://服务地址:8080/sign
```

### 请求参数

所有参数放在 `params` 对象中，参数名和值与星驿支付官方文档一致。

### 示例：付款码支付签名

```json
{
  "params": {
    "agetId": "机构号",
    "custId": "商户号",
    "orderNo": "商户订单号",
    "txamt": "100",
    "code": "用户付款码",
    "tradingIp": "终端IP",
    "type": "A",
    "timeStamp": "20240101120000",
    "version": "1.0.0"
  }
}
```

### 示例：JSAPI支付签名

```json
{
  "params": {
    "agetId": "机构号",
    "custId": "商户号",
    "orderNo": "商户订单号",
    "txamt": "100",
    "openid": "用户openid",
    "payWay": "1",
    "ip": "用户IP",
    "wxAppid": "微信appid",
    "timeStamp": "20240101120000",
    "version": "1.0.0"
  }
}
```

### 示例：订单查询签名

```json
{
  "params": {
    "agetId": "机构号",
    "custId": "商户号",
    "orderNo": "商户订单号",
    "orderTime": "20240101",
    "timeStamp": "20240101120000",
    "version": "1.0.0"
  }
}
```

### 示例：退款签名

```json
{
  "params": {
    "agetId": "机构号",
    "custId": "商户号",
    "orderNo": "退款订单号",
    "refundAmount": "100",
    "tag": "1",
    "reOrderNo": "原星驿订单号",
    "timeStamp": "20240101120000",
    "version": "1.0.0"
  }
}
```

### 响应结果

```json
{
  "code": 0,
  "message": "success",
  "data": {
    "sign": "签名结果"
  }
}
```

拿到 `sign` 后，将其加入请求参数中，然后请求星驿支付官方API。

---

## 二、验签接口

### 接口地址
```
POST http://服务地址:8080/verify
```

### 请求参数

将星驿支付异步通知的**完整参数**传入，参数名保持原样（大写下划线格式）。

### 示例

```json
{
  "params": {
    "AGET_ID": "机构号",
    "CUST_ID": "商户号",
    "ORDER_NO": "星驿订单号",
    "THREE_ORDER_NO": "商户订单号",
    "TXAMT": "100",
    "ORDER_STATUS": "1",
    "PAY_WAY": "1",
    "PAY_CHANNEL": "2",
    "OPEN_ID": "用户openid",
    "sign": "星驿传来的签名"
  }
}
```

### 响应结果

```json
{
  "code": 0,
  "message": "success",
  "data": {
    "valid": true
  }
}
```

- `valid: true` - 签名有效，可信任该通知
- `valid: false` - 签名无效，可能是伪造请求

---

## 三、各支付方式参数说明

### 公共必填参数

| 参数名 | 说明 | 示例 |
|--------|------|------|
| agetId | 机构号 | 由星驿支付提供 |
| custId | 商户号 | 由星驿支付提供 |
| orderNo | 商户订单号 | 自行生成，需唯一 |
| txamt | 金额 | 单位：分，如"100"表示1元 |
| timeStamp | 时间戳 | 格式：yyyyMMddHHmmss |
| version | 版本号 | 固定"1.0.0" |

### 付款码支付特有参数

| 参数名 | 说明 | 示例 |
|--------|------|------|
| code | 用户付款码 | 微信18位/支付宝16-24位/银联19位 |
| tradingIp | 终端IP | 如"192.168.1.1" |
| type | 设备类型 | P-智能POS/A-app扫码/C-PC端/T-台牌扫码 |

### JSAPI支付特有参数

| 参数名 | 说明 | 示例 |
|--------|------|------|
| openid | 用户openid | 微信openid/支付宝userId |
| payWay | 支付方式 | 1-微信/2-支付宝/3-银联/4-京东白条 |
| ip | 用户IP | 用户手机公网IP |
| wxAppid | 微信appid | 微信支付必传 |

### 订单查询特有参数

| 参数名 | 说明 | 示例 |
|--------|------|------|
| orderNo | 商户订单号 | 与gtOrderNo二选一 |
| gtOrderNo | 星驿订单号 | 与orderNo二选一 |
| orderTime | 订单日期 | 格式：yyyyMMdd |

### 退款特有参数

| 参数名 | 说明 | 示例 |
|--------|------|------|
| refundAmount | 退款金额 | 单位：分 |
| tag | 订单类型 | 1-支付宝/2-微信/9-银联/11-银行卡/12-数币 |
| reOrderNo | 原星驿订单号 | 支付时返回的orderNo |

---

## 四、完整使用流程

### 发起支付流程

```
1. 构造请求参数
   ↓
2. 调用本服务 /sign 接口获取签名
   ↓
3. 将签名加入参数的 sign 字段
   ↓
4. 自行请求星驿支付官方API
   ↓
5. 处理返回结果
```

### 接收异步通知流程

```
1. 收到星驿支付的异步通知
   ↓
2. 调用本服务 /verify 接口验证签名
   ↓
3. 如果 valid=true，处理业务逻辑
   ↓
4. 返回 {"rspCod":"000000","rspMsg":"success"}
```
