﻿/*
* Copyright (C) 2021 Duowan Inc. All rights reserved.
*
* Licensed under the Apache License, Version 2.0 (the "License");
* you may not use this file except in compliance with the License.
* You may obtain a copy of the License at
*
*    http://www.apache.org/licenses/LICENSE-2.0
*
* Unless required by applicable law or agreed to in writing,
* software distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific language governing permissions and
* limitations under the License.
*/

#ifndef __X_PACK_JSON_DECODER_H
#define __X_PACK_JSON_DECODER_H

#include <fstream>

#include "rapidjson_custom.h"
#include "thirdparty/rapidjson/document.h"
#include "thirdparty/rapidjson/error/en.h"

#include "xdecoder.h"


namespace xpack {

class JsonDecoder:public XDecoder<JsonDecoder>, private noncopyable {
    friend class XDecoder<JsonDecoder>;
    friend class JsonData;
    class MemberIterator {
        friend class JsonDecoder;
    public:
        MemberIterator(const rapidjson::Value::ConstMemberIterator iter, JsonDecoder* parent):_iter(iter),_parent(parent){}
        bool operator != (const MemberIterator &that) const {
            return _iter != that._iter;
        }
        MemberIterator& operator ++ () {
            ++_iter;
            return *this;
        }
        const char *Key() const {
            return _iter->name.GetString();
        }
        JsonDecoder& Val() const {
            return _parent->member(*this, *(_parent->alloc()));
        }
    private:
        rapidjson::Value::ConstMemberIterator _iter;
        JsonDecoder* _parent;
    };
public:
    using xdoc_type::decode;
    typedef MemberIterator Iterator;

    JsonDecoder(const std::string& str, bool isfile=false):xdoc_type(NULL, ""),_doc(new rapidjson::Document),_val(_doc) {
        std::string err;
        std::string data;

        do {
            const unsigned int parseFlags = rapidjson::kParseNanAndInfFlag;
            if (isfile) {
                std::ifstream fs(str.c_str(), std::ifstream::binary);
                if (!fs) {
                    err = "Open file["+str+"] fail.";
                    break;
                }
                std::string _tmp((std::istreambuf_iterator<char>(fs)), std::istreambuf_iterator<char>());
                data.swap(_tmp);
                _doc->Parse<parseFlags>(data);
            } else  {
                _doc->Parse<parseFlags>(str);
            }

            if (_doc->HasParseError()) {
                size_t offset = _doc->GetErrorOffset();
                std::string parse_err(rapidjson::GetParseError_En(_doc->GetParseError()));
                if  (isfile) {
                    std::string err_data = data.substr(offset, 32);
                    err = "Parse json file \""+str+"\" fail. err="+parse_err+". offset="+err_data;
                    break;
                } else {
                    std::string err_data = str.substr(offset, 32);
                    err = "Parse json string fail. err="+parse_err+". offset="+err_data;
                    break;
                }
            }

            return;
        } while (false);

        delete _doc;
        _doc = NULL;
        throw std::runtime_error(err);
    }

    JsonDecoder(const rapidjson::Value*v):xdoc_type(NULL, ""),_doc(NULL),_val(v) {
    }

    ~JsonDecoder() {
        if (NULL != _doc) {
            delete _doc;
            _doc = NULL;
        }
    }

    inline const char * Type() const {
        return "json";
    }

public:
    // decode
    #define XPACK_JSON_DECODE(nullVal, f, ...)                  \
        bool isNull;                                            \
        const rapidjson::Value *v = get_val(key, isNull);       \
        bool ret = false;                                       \
        if (NULL != v) {                                        \
            try {                                               \
                val = __VA_ARGS__ v->f();                       \
            } catch (const std::exception&e) {                  \
                (void)e;                                        \
                decode_exception("type unmatch", key);          \
            }                                                   \
            ret = true;                                         \
        } else if (isNull) {                                    \
            val = nullVal;                                      \
            if (0==(Extend::CtrlFlag(ext)&X_PACK_CTRL_FLAG_IGNORE_NULL)) {\
                ret = true;                                     \
            }                                                   \
        } else if (NULL!=key && Extend::Mandatory(ext)) {       \
            decode_exception("mandatory key not found", key);   \
        }                                                       \
        return ret


    bool decode(const char*key, std::string &val, const Extend *ext) {
        bool isNull;
        const rapidjson::Value *v = get_val(key, isNull);
        if (NULL != v) {
            try {
                val = std::string(v->GetString(), v->GetStringLength());
            } catch (const std::exception&e) {
                (void)e;
                decode_exception("type unmatch", key);
            }
            return true;
        } else if (isNull) {
            return true;
        } else if (NULL!=key && Extend::Mandatory(ext)) {
            decode_exception("mandatory key not found", key);
        }
        return false;
    }
    bool decode(const char*key, bool &val, const Extend *ext) {
        bool isNull;
        const rapidjson::Value *v = get_val(key, isNull);
        if (isNull) {
            val = false;
            return true;
        } else if (NULL == v) {
            if (NULL!=key && Extend::Mandatory(ext)) {
                decode_exception("mandatory key not found", key);
            }
            return false;
        } else if (v->IsBool()) {
            val = v->GetBool();
            return true;
        } else if (v->IsInt64()) {
            val = (0 != (v->GetInt64()));
            return true;
        } else {
            decode_exception("wish bool, but not bool or int", key);
            return false;
        }
    }
    bool decode(const char*key, char &val, const Extend *ext) {
        XPACK_JSON_DECODE('\0', GetInt, (char));
    }
    bool decode(const char*key, signed char &val, const Extend *ext) {
        XPACK_JSON_DECODE(0, GetInt, (char));
    }
    bool decode(const char*key, unsigned char &val, const Extend *ext) {
        XPACK_JSON_DECODE(0, GetInt, (unsigned char));
    }
    bool decode(const char*key, short &val, const Extend *ext) {
        XPACK_JSON_DECODE(0, GetInt, (short));
    }
    bool decode(const char*key, unsigned short &val, const Extend *ext) {
        XPACK_JSON_DECODE(0, GetInt, (unsigned short));
    }
    bool decode(const char*key, int &val, const Extend *ext) {
        XPACK_JSON_DECODE(0, GetInt);
    }
    bool decode(const char*key, unsigned int &val, const Extend *ext) {
        XPACK_JSON_DECODE(0, GetUint);
    }
    bool decode(const char*key, long &val, const Extend *ext) {
        XPACK_JSON_DECODE(0, GetInt64, (long));
    }
    bool decode(const char*key, unsigned long &val, const Extend *ext) {
       XPACK_JSON_DECODE(0, GetUint64, (unsigned long));
    }
    bool decode(const char*key, long long &val, const Extend *ext) {
        XPACK_JSON_DECODE(0, GetInt64, (long long));
    }
    bool decode(const char*key, unsigned long long &val, const Extend *ext) {
       XPACK_JSON_DECODE(0, GetUint64, (unsigned long long));
    }
    bool decode(const char*key, float &val, const Extend *ext) {
        XPACK_JSON_DECODE(0, GetFloat);
    }
    bool decode(const char*key, double &val, const Extend *ext) {
        XPACK_JSON_DECODE(0, GetDouble);
    }
    bool decode(const char*key, long double &val, const Extend *ext) {
        XPACK_JSON_DECODE(0, GetDouble, (long double));
    }

    // map<int, T> xml not support use number as label.
    // So this function is defined here instead of xdecoder.h
    template <class K, class T>
    typename x_enable_if<numeric<K>::is_integer, bool>::type decode(const char*key, std::map<K,T>& val, const Extend *ext) {
        return decode_map<std::map<K,T>, K, T>(key, val, ext, Util::atoi);
    }

    #ifdef X_PACK_SUPPORT_CXX0X
    // enum is_enum implementation is too complicated, so not support in c++03
    template <class K, class T>
    typename x_enable_if<std::is_enum<K>::value, bool>::type decode(const char*key, std::map<K,T>& val, const Extend *ext) {
        return decode_map<std::map<K,T>, K, T>(key, val, ext, Util::atoi);
    }
    #endif

    #ifdef XPACK_SUPPORT_QT
    template <class K, class T>
    typename x_enable_if<numeric<K>::is_integer, bool>::type decode(const char*key, QMap<K,T>& val, const Extend *ext) {
        return decode_map<QMap<K,T>, K, T>(key, val, ext, Util::atoi);
    }
    #endif

    // array
    size_t Size() {
        if (_val->IsArray()) {
            return (size_t)_val->Size();
        } else {
            return 0;
        }
    }

    JsonDecoder& operator[](size_t index) {
        JsonDecoder *d = alloc();
        member(index, *d, NULL);
        return *d;
    }

    JsonDecoder& operator[](const char*key) {
        JsonDecoder *d = alloc();
        member(key, *d, NULL);
        return *d;
    }

    // iter
    Iterator Begin() {
        return Iterator(_val->MemberBegin(), this);
    }
    Iterator End() {
        return Iterator(_val->MemberEnd(), this);
    }
    operator bool() const {
        return NULL != _val;
    }

private:
    JsonDecoder():xdoc_type(NULL, ""),_doc(NULL),_val(NULL) {
    }

    JsonDecoder& member(size_t index, JsonDecoder&d, const Extend *ext) const {
        (void)ext;
        if (NULL != _val && _val->IsArray()) {
            if (index < (size_t)_val->Size()) {
                d.init_base(this, index);
                d._val = &(*_val)[(rapidjson::SizeType)index];
            } else {
                decode_exception("Out of index", NULL);
            }
        } else {
            decode_exception("not array", NULL);
        }

        return d;
    }

    JsonDecoder& member(const char*key, JsonDecoder&d, const Extend *ext) const {
        (void)ext;
        if (NULL != _val && _val->IsObject()) {
            rapidjson::Value::ConstMemberIterator iter;
            if (_val->MemberEnd()!=(iter=_val->FindMember(key)) && !(iter->value.IsNull())) {
                d.init_base(this, key);
                d._val = &(iter->value);
            }
        } else {
            decode_exception("not object", key);
        }

        return d;
    }

    JsonDecoder& member(const Iterator &iter, JsonDecoder&d) const {
        d.init_base(iter._parent, iter._iter->name.GetString());
        d._val = &(iter._iter->value);
        return d;
    }

    const rapidjson::Value* get_val(const char *key, bool &isNull) {
        isNull = false;
        if (NULL == key) {
            return _val;
        } else if (NULL != _val) {
            rapidjson::Value::ConstMemberIterator iter = _val->FindMember(key);
            if (iter != _val->MemberEnd()) {
                if (!(iter->value.IsNull())) {
                    return &iter->value;
                } else {
                    isNull = true;
                    return NULL;
                }
            } else {
                return NULL;
            }
        } else {
            return NULL;
        }
    }

    rapidjson::Document* _doc;
    const rapidjson::Value* _val;
};


}

#endif
