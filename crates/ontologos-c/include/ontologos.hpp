#pragma once

#include "ontologos.h"

#include <stdexcept>
#include <string>
#include <utility>

namespace ontologos {

class Error : public std::runtime_error {
 public:
  explicit Error(const std::string &message) : std::runtime_error(message) {}
};

inline void throw_if_error() {
  const char *code = ontologos_last_error_code();
  if (!code) {
    return;
  }
  const char *message = ontologos_last_error_message();
  std::string text = message ? message : code;
  ontologos_clear_last_error();
  throw Error(text);
}

inline int64_t check_handle(int64_t handle, const char *what) {
  if (handle == 0) {
    throw_if_error();
    throw Error(std::string("failed to create ") + what);
  }
  return handle;
}

inline std::string take_string(char *ptr, bool allow_null = false) {
  if (!ptr) {
    if (!allow_null) {
      throw_if_error();
    }
    return {};
  }
  std::string value(ptr);
  ontologos_string_free(ptr);
  return value;
}

class Ontology {
 public:
  Ontology() : handle_(0) {}
  explicit Ontology(int64_t handle) : handle_(check_handle(handle, "ontology")) {}
  Ontology(const Ontology &) = delete;
  Ontology &operator=(const Ontology &) = delete;
  Ontology(Ontology &&other) noexcept : handle_(std::exchange(other.handle_, 0)) {}
  Ontology &operator=(Ontology &&other) noexcept {
    if (this != &other) {
      close();
      handle_ = std::exchange(other.handle_, 0);
    }
    return *this;
  }
  ~Ontology() { close(); }

  static Ontology from_json(const std::string &json) {
    return Ontology(ontologos_ontology_from_json(json.c_str()));
  }

  std::string to_json() const {
    ensure_open();
    return take_string(ontologos_ontology_to_json(handle_));
  }

  int64_t axiom_count() const {
    ensure_open();
    const int64_t count = ontologos_ontology_axiom_count(handle_);
    if (count < 0) {
      throw_if_error();
    }
    return count;
  }

  int64_t native_handle() const {
    ensure_open();
    return handle_;
  }

 private:
  void ensure_open() const {
    if (handle_ == 0) {
      throw Error("ontology handle is closed");
    }
  }

  void close() {
    if (handle_ != 0) {
      ontologos_ontology_close(handle_);
      handle_ = 0;
    }
  }

  int64_t handle_;
};

class OntologyBuilder {
 public:
  OntologyBuilder() : handle_(check_handle(ontologos_builder_new(), "ontology builder")) {}
  OntologyBuilder(const OntologyBuilder &) = delete;
  OntologyBuilder &operator=(const OntologyBuilder &) = delete;
  OntologyBuilder(OntologyBuilder &&other) noexcept : handle_(std::exchange(other.handle_, 0)) {}
  OntologyBuilder &operator=(OntologyBuilder &&other) noexcept {
    if (this != &other) {
      close();
      handle_ = std::exchange(other.handle_, 0);
    }
    return *this;
  }
  ~OntologyBuilder() { close(); }

  OntologyBuilder &add_class(const std::string &iri) {
    ensure_open();
    handle_ = ontologos_builder_add_class(handle_, iri.c_str());
    return *this;
  }

  OntologyBuilder &subclass_of(const std::string &subclass, const std::string &superclass) {
    ensure_open();
    handle_ = ontologos_builder_subclass_of(handle_, subclass.c_str(), superclass.c_str());
    return *this;
  }

  Ontology build() {
    ensure_open();
    const int64_t ontology = ontologos_builder_build(handle_);
    handle_ = 0;
    return Ontology(ontology);
  }

 private:
  void ensure_open() const {
    if (handle_ == 0) {
      throw Error("ontology builder handle is closed");
    }
  }

  void close() {
    if (handle_ != 0) {
      ontologos_builder_close(handle_);
      handle_ = 0;
    }
  }

  int64_t handle_;
};

class Reasoner {
 public:
  Reasoner() : handle_(0) {}
  Reasoner(const Ontology &ontology, const char *profile = "el")
      : handle_(check_handle(
            ontologos_reasoner_new(ontology.native_handle(), profile, 0, -1),
            "reasoner")) {}
  Reasoner(const Reasoner &) = delete;
  Reasoner &operator=(const Reasoner &) = delete;
  Reasoner(Reasoner &&other) noexcept : handle_(std::exchange(other.handle_, 0)) {}
  Reasoner &operator=(Reasoner &&other) noexcept {
    if (this != &other) {
      close();
      handle_ = std::exchange(other.handle_, 0);
    }
    return *this;
  }
  ~Reasoner() { close(); }

  std::string classify() const {
    ensure_open();
    return take_string(ontologos_reasoner_classify(handle_));
  }

  Reasoner &add_subclass_of(const std::string &subclass, const std::string &superclass) {
    ensure_open();
    handle_ = ontologos_reasoner_add_subclass_of(handle_, subclass.c_str(), superclass.c_str());
    return *this;
  }

 private:
  void ensure_open() const {
    if (handle_ == 0) {
      throw Error("reasoner handle is closed");
    }
  }

  void close() {
    if (handle_ != 0) {
      ontologos_reasoner_close(handle_);
      handle_ = 0;
    }
  }

  int64_t handle_;
};

inline std::string version() { return take_string(ontologos_version()); }

}  // namespace ontologos
