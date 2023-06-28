#ifndef RESULT_H
#define RESULT_H

template <typename T>
class Result {
public:
    static Result<T> ok(T value) {
        Result<T> result;
        result.m_value = value;
        result.isOk = true;
        return result;
    }

    static Result<T> error(const char* msg) {
        Result<T> result;
        result.errorMsg = msg;
        result.isOk = false;
        return result;
    }

    bool ok() {
        return this->isOk;
    }

    T value() {
        return this->m_value;
    }

    const char* error() {
        return this->errorMsg;
    }

private:
    Result() {}
    T m_value;
    const char* errorMsg;
    bool isOk;
};

#endif