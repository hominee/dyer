-- Your SQL goes here

-- Sqlite
/*
 *CREATE TABLE quotes (
 *  id BIGINT NOT NULL PRIMARY KEY,
 *  role VARCHAR NOT NULL,
 *  text TEXT NOT NULL,
 *  author VARCHAR NOT NULL,
 *  tags TEXT
 *);
 */


-- PostgreSQL
/*
 *CREATE TABLE quotes (
 *  id BIGINT PRIMARY KEY,
 *  role Roles NOT NULL,
 *  text TEXT NOT NULL,
 *  author VARCHAR (64) NOT NULL,
 *  tags json
 *);
 */

-- Mysql
CREATE TABLE quotes (
	id BIGINT PRIMARY KEY,
	role VARCHAR(10) NOT NULL,
	text TEXT NOT NULL,
	author VARCHAR (64) NOT NULL,
	tags JSON
);

